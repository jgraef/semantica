use std::{
    marker::PhantomData,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use eventsource_stream::{
    Event,
    EventStreamError,
};
use futures::{
    stream::Stream,
    StreamExt,
};
use serde::Deserialize;

use super::Error;

pub struct EventStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<Event, EventStreamError<reqwest::Error>>> + 'static>>,
    _t: PhantomData<T>,
}

impl<T> Unpin for EventStream<T> {}

impl<T> EventStream<T> {
    pub fn new(
        stream: impl Stream<Item = Result<Event, EventStreamError<reqwest::Error>>> + 'static,
    ) -> Self {
        Self {
            inner: Box::pin(stream),
            _t: PhantomData,
        }
    }
}

impl<T: for<'de> Deserialize<'de>> Stream for EventStream<T> {
    type Item = Result<T, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error.into()))),
            Poll::Ready(Some(Ok(event))) => {
                let result = serde_json::from_str(&event.data).map_err(|e| Error::EventJson(e));
                Poll::Ready(Some(result))
            }
        }
    }
}
