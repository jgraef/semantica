#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("client")]
    Client(#[from] semantica_client::Error),
}
