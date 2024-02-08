use std::{
    fmt::Display,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use futures::Future;
use pin_project::pin_project;

#[pin_project]
pub struct LogAndDiscardErrorFuture<F> {
    #[pin]
    inner: F,
}

impl<F: Future<Output = Result<(), E>>, E: std::error::Error> Future
    for LogAndDiscardErrorFuture<F>
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(()),
            Poll::Ready(Err(error)) => {
                let mut error: &dyn std::error::Error = &error;

                log::error!("{error}");

                while let Some(source) = error.source() {
                    log::error!(" - {source}");
                    error = source;
                }

                Poll::Ready(())
            }
        }
    }
}

pub trait LogAndDiscardErrorExt: Sized {
    fn log_and_discard_error(self) -> LogAndDiscardErrorFuture<Self> {
        LogAndDiscardErrorFuture { inner: self }
    }
}

impl<F: Future<Output = Result<(), E>>, E: Display> LogAndDiscardErrorExt for F {}
