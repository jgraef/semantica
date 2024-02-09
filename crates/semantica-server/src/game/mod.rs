pub mod ai;
pub mod inventory;
pub mod node;

use std::{
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::Request,
    http::StatusCode,
    response::Html,
    Router,
    ServiceExt,
};
use chrono::{
    DateTime,
    NaiveDateTime,
    Utc,
};
use semantica_protocol::auth::AuthSecret;
use serde::{
    Deserialize,
    Serialize,
};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::{
    PgConnection,
    PgPool,
    Postgres,
};
use tokio::{
    net::TcpListener,
    signal,
    task::AbortHandle,
};
use tower_http::{
    classify::{
        ServerErrorsAsFailures,
        SharedClassifier,
    },
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
use uuid::{
    uuid,
    Uuid,
};

use crate::{
    api::auth::hash_auth_secret,
    error::Error,
    game::{
        ai::Ai,
        node::{
            create_node_content,
            create_root_node,
        },
    },
};

#[derive(Debug)]
struct Inner {
    pool: PgPool,
    ai: Ai,
}

#[derive(Clone, Debug)]
pub struct Game {
    inner: Arc<Inner>,
}

impl Game {
    pub async fn new(pool: PgPool, secrets: SecretStore) -> Result<Self, Error> {
        sqlx::migrate!().run(&pool).await?;

        let ai = Ai::new(&secrets);

        let this = Self {
            inner: Arc::new(Inner { pool, ai }),
        };

        this.initialize().await?;

        Ok(this)
    }

    pub async fn transaction(&self) -> Result<Transaction, Error> {
        let transaction = self.inner.pool.begin().await?;
        Ok(Transaction {
            id: Uuid::new_v4(),
            game: self.clone(),
            transaction,
            now: Utc::now(),
        })
    }

    pub fn ai(&self) -> &Ai {
        &self.inner.ai
    }

    pub fn pool(&self) -> &PgPool {
        &self.inner.pool
    }

    async fn initialize(&self) -> Result<(), Error> {
        const INITIALIZED: Uuid = uuid!("1c02e958-b74c-48f3-97e8-a7d5a8f53703");

        let mut transaction = self.transaction().await?;

        let is_initialized = transaction.get_property::<bool>(INITIALIZED).await?;

        if !is_initialized {
            transaction.initialize_game_state().await?;

            transaction.set_property(Some(INITIALIZED), &true).await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn serve(self, address: SocketAddr) -> Result<(), Error> {
        let (session_layer, session_layer_task_abort_handle) =
            session_layer(self.inner.pool.clone()).await?;

        let router = Router::new()
            .nest("/api/v1", crate::api::routes())
            .fallback(not_found)
            .layer(session_layer)
            .layer(log_layer())
            .with_state(self);

        // run server until a shutdown signal is received
        axum::serve(
            TcpListener::bind(address).await?,
            ServiceExt::<Request>::into_make_service(
                NormalizePathLayer::trim_trailing_slash().layer(router),
            ),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        // abort session layer cleanup task
        session_layer_task_abort_handle.abort();

        Ok(())
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Game {
    /// Takes the router that is returned by the user in their
    /// [shuttle_runtime::main] function and binds to an address passed in
    /// by shuttle.
    async fn bind(self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        self.serve(addr)
            .await
            .map_err(CustomError::new)
            .map_err(Into::into)
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

async fn session_layer(
    pool: PgPool,
) -> Result<(SessionManagerLayer<PostgresStore>, AbortHandle), Error> {
    let session_store = PostgresStore::new(pool)
        .with_schema_name("public")
        .unwrap()
        .with_table_name("sessions")
        .unwrap();

    session_store.migrate().await?;

    let session_deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnSessionEnd);

    Ok((session_layer, session_deletion_task.abort_handle()))
}

fn log_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

async fn not_found() -> (StatusCode, Html<&'static str>) {
    (StatusCode::NOT_FOUND, Html("<h1>Not Found</h1>"))
}

pub struct Transaction<'a> {
    id: Uuid,
    game: Game,
    transaction: sqlx::Transaction<'a, Postgres>,
    now: DateTime<Utc>,
}

impl<'a> Transaction<'a> {
    pub async fn commit(self) -> Result<(), Error> {
        self.transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), Error> {
        self.transaction.rollback().await?;
        Ok(())
    }

    pub fn db(&mut self) -> &mut PgConnection {
        &mut *self.transaction
    }

    pub async fn get_property<T: for<'de> Deserialize<'de>>(
        &mut self,
        key: Uuid,
    ) -> Result<T, Error> {
        let row = sqlx::query!("SELECT value FROM properties WHERE key = $1", key)
            .fetch_one(self.db())
            .await?;
        let value = serde_json::from_value(row.value)?;
        Ok(value)
    }

    pub async fn set_property<T: Serialize>(
        &mut self,
        key: Option<Uuid>,
        value: &T,
    ) -> Result<Uuid, Error> {
        let key = key.unwrap_or_else(|| Uuid::new_v4());
        let value = serde_json::to_value(value)?;
        sqlx::query!(
            "INSERT INTO properties VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2",
            key,
            value
        )
        .execute(self.db())
        .await?;
        Ok(key)
    }

    pub fn now(&self) -> DateTime<Utc> {
        self.now
    }

    pub fn now_naive(&self) -> NaiveDateTime {
        self.now.naive_utc()
    }

    pub async fn initialize_game_state(&mut self) -> Result<(), Error> {
        tracing::info!("initializing game state");

        // create root node
        let content = create_node_content(
            "This is the beginning of your story.\nThere's nothing here yet, besides these words.",
        );
        let root_node = create_root_node(content);
        self.insert_node(&root_node).await?;

        // create debug user
        let auth_secret = AuthSecret("1UePwNhkVj_MyoE7wfXlBgCH6zncFLYv".to_owned().into());
        sqlx::query!(
            r#"
            INSERT INTO users (
                user_id,
                name,
                auth_secret,
                in_node,
                god_mode
            ) VALUES (
                '43d65ac1-2778-49e8-b28d-65c7334cec32',
                'test',
                $1,
                $2,
                true
            );
            "#,
            hash_auth_secret(auth_secret).await,
            root_node.node_id.0
        )
        .execute(self.db())
        .await?;

        Ok(())
    }
}
