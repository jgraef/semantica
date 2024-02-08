pub mod ai;
pub mod api;
pub mod context;
pub mod error;

use std::net::SocketAddr;

use axum::{
    extract::Request,
    routing::get,
    Router,
    ServiceExt,
};
use context::Context;
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use tokio::{
    signal,
    task::AbortHandle,
};
use tower_http::{
    normalize_path::NormalizePathLayer,
    trace::{
        DefaultMakeSpan,
        DefaultOnResponse,
        TraceLayer,
    },
};
use tower_layer::Layer;
use tower_sessions::{
    ExpiredDeletion,
    Expiry,
    SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::Level;

/// A wrapper type for [axum::Router] so we can implement
/// [shuttle_runtime::Service] for it.
pub struct AxumService {
    pub router: Router,
    pub abort_handles: Vec<AbortHandle>,
}

impl AxumService {
    pub fn with_abort_handle(mut self, abort_handle: AbortHandle) -> Self {
        self.abort_handles.push(abort_handle);
        self
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for AxumService {
    /// Takes the router that is returned by the user in their
    /// [shuttle_runtime::main] function and binds to an address passed in
    /// by shuttle.
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let app = NormalizePathLayer::trim_trailing_slash().layer(self.router);

        // run server until a shutdown signal is received
        axum::serve(
            shuttle_runtime::tokio::net::TcpListener::bind(addr)
                .await
                .map_err(CustomError::new)?,
            ServiceExt::<Request>::into_make_service(app),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(CustomError::new)?;

        // abort all registered tasks
        for abort_handle in self.abort_handles {
            abort_handle.abort();
        }

        Ok(())
    }
}

impl From<Router> for AxumService {
    fn from(router: Router) -> Self {
        Self {
            router,
            abort_handles: vec![],
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://semantica:{secrets.DB_PASSWORD}@localhost/semantica"
    )]
    pool: PgPool,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> Result<AxumService, shuttle_runtime::Error> {
    let context = Context::new(pool.clone(), &secrets)
        .await
        .map_err(CustomError::new)?;

    let session_store = PostgresStore::new(pool);
    session_store.migrate().await.map_err(CustomError::new)?;
    let session_deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnSessionEnd);

    let router = Router::new()
        .nest("/api/v1", crate::api::routes())
        .fallback(|| async { "not found" })
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(context);

    let service = AxumService::from(router).with_abort_handle(session_deletion_task.abort_handle());

    Ok(service)
}
