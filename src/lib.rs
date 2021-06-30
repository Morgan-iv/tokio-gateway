use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Gateway {
    data: Vec<u8>,
    pos: usize,
    is_closed: bool,
    waker: Option<Waker>,
}

impl Gateway {
    pub fn new() -> Self {
        Gateway {
            data: Vec::new(),
            pos: 0,
            is_closed: false,
            waker: None,
        }
    }

    fn close_write(&mut self) {
        self.is_closed = true;
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    pub fn as_ref(&self) -> Option<&Vec<u8>> {
        match self.is_closed {
            true => Some(&self.data),
            false => None,
        }
    }
}

impl AsyncRead for Gateway {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match (self.is_closed, self.pos < self.data.len()) {
            (true, true) => {
                let len = (self.data.len() - self.pos).min(buf.remaining());
                buf.put_slice(&self.data[self.pos..self.pos + len]);
                self.pos += len;
                Poll::Ready(Ok(()))
            }
            (true, false) => Poll::Ready(Ok(())),
            (false, _) => {
                self.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl AsyncWrite for Gateway {
    fn poll_write(mut self: Pin<&mut Self>, _: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        if self.is_closed {
            Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()))
        } else {
            self.data.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        self.close_write();
        Poll::Ready(Ok(()))
    }
}
