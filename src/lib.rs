use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;

pub struct Gateway {
    data: Vec<u8>,
    is_closed: bool,
}

impl Gateway {
    pub fn new() -> Self {
        Gateway {
            data: Vec::new(),
            is_closed: false,
        }
    }

    pub fn as_ref(&self) -> Option<&Vec<u8>> {
        match self.is_closed {
            true => Some(&self.data),
            false => None,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut Vec<u8>> {
        match self.is_closed {
            true => Some(&mut self.data),
            false => None,
        }
    }
}

impl From<Gateway> for Vec<u8> {
    fn from(value: Gateway) -> Self {
        value.data
    }
}

impl AsyncWrite for Gateway {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
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
        self.is_closed = true;
        Poll::Ready(Ok(()))
    }
}

impl Default for Gateway {
    fn default() -> Self {
        Self::new()
    }
}
