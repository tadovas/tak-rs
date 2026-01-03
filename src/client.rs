use crate::protocol::xml::CotLegacyCodec;
use crate::protocol::CodecError;
use futures::{pin_mut, StreamExt};
use std::io::ErrorKind;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::Framed;
use tracing::info;

fn unexpected_eof_is_none<V>(res: Option<Result<V, CodecError>>) -> Option<Result<V, CodecError>> {
    match res {
        // because of https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
        Some(Err(CodecError::Io(e))) if e.kind() == ErrorKind::UnexpectedEof => None,
        res => res,
    }
}

pub struct CotConnection<T> {
    io_stream: T,
    connection_id: String,
}

impl<T> CotConnection<T> {
    pub fn new(io_stream: T, connection_id: String) -> Self {
        Self {
            io_stream,
            connection_id,
        }
    }
}

impl<T: AsyncRead + AsyncWrite> CotConnection<T> {
    pub async fn conn_loop(self) -> anyhow::Result<()> {
        let frames = Framed::new(self.io_stream, CotLegacyCodec::new(4 * 1024));
        pin_mut!(frames);

        while let Some(res) = unexpected_eof_is_none(frames.next().await) {
            let message = res?;
            info!("{message:#?}");
        }
        info!("{} - disconnected", self.connection_id);
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

    impl AsyncWrite for UnexpectedEOFReader {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
            unimplemented!()
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
            unimplemented!()
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), io::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_client_disconnection_without_err() {
        //FIXME - need a guard against infinite loop
        let client_conn = CotConnection::new(UnexpectedEOFReader, "test conn".into());
        let res = client_conn.conn_loop().await;
        assert!(res.is_ok())
    }
}
