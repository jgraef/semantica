use axum::{
    extract::State,
    response::{
        sse,
        Sse,
    },
};
use futures::{
    Stream,
    StreamExt,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    context::Context,
    error::Error,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    // todo
}

pub async fn subscribe(
    State(_context): State<Context>,
) -> Sse<impl Stream<Item = Result<sse::Event, Error>>> {
    // todo: subscribe to events
    let stream = futures::stream::empty::<Event>();

    let stream = stream.map(|event| Ok(sse::Event::default().json_data(event)?));

    Sse::new(stream)
}
