use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;
use tokio::sync::oneshot::{channel, Receiver, Sender};

pub struct Gateway {
    inner: Option<GatewayInner>,
}

struct GatewayInner {
    data: Vec<u8>,
    tx: Sender<Vec<u8>>,
}

impl Gateway {
    pub fn new() -> (Self, Receiver<Vec<u8>>) {
        let (tx, rx) = channel();
        (
            Gateway {
                inner: Some(GatewayInner {
                    data: Vec::new(),
                    tx,
                }),
            },
            rx,
        )
    }

    pub fn as_ref(&self) -> Option<&Vec<u8>> {
        self.inner.as_ref().map(|i| &i.data)
    }

    pub fn as_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.inner.as_mut().map(|i| &mut i.data)
    }
}

impl AsyncWrite for Gateway {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        match self.inner.as_mut() {
            Some(i) => {
                i.data.extend_from_slice(buf);
                Poll::Ready(Ok(buf.len()))
            }
            None => Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into())),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(
            self.inner
                .take()
                .and_then(|i| i.tx.send(i.data).ok())
                .ok_or_else(|| std::io::ErrorKind::BrokenPipe.into()),
        )
    }
}
