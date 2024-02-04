use crate::protocol::xml::CotLegacyCodec;
use crate::protocol::CodecError;
use crate::router::command::Commands;
use futures::{pin_mut, StreamExt};
use std::io::ErrorKind;
use tokio::io::AsyncRead;
use tokio_util::codec::FramedRead;
use tracing::info;

fn unexpected_eof_is_none<V>(res: Option<Result<V, CodecError>>) -> Option<Result<V, CodecError>> {
    match res {
        // because of https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
        Some(Err(CodecError::Io(e))) if e.kind() == ErrorKind::UnexpectedEof => None,
        res => res,
    }
}

pub(super) struct ClientConnection<T, C> {
    io_stream: T,
    _commands: C,
}

impl<T, C> ClientConnection<T, C> {
    pub(super) fn new(io_stream: T, commands: C) -> Self {
        Self {
            io_stream,
            _commands: commands,
        }
    }
}

impl<T: AsyncRead, C: Commands> ClientConnection<T, C> {
    pub(super) async fn conn_loop(self) -> anyhow::Result<()> {
        let frames = FramedRead::new(self.io_stream, CotLegacyCodec::new(4 * 1024));
        pin_mut!(frames);

        while let Some(res) = unexpected_eof_is_none(frames.next().await) {
            let message = res?;
            info!("{message:#?}");
        }
        info!("Disconnected");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io;
    use std::io::ErrorKind;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, ReadBuf};

    struct UnexpectedEOFReader;
    impl AsyncRead for UnexpectedEOFReader {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(ErrorKind::UnexpectedEof, "test error")))
        }
    }

    struct TestCommands;

    impl Commands for TestCommands {}

    #[tokio::test]
    async fn test_client_disconnection_without_err() {
        //FIXME - need a guard against infinite loop
        let client_conn = ClientConnection::new(UnexpectedEOFReader, TestCommands);
        let res = client_conn.conn_loop().await;
        assert!(res.is_ok())
    }
}
