use crate::protocol::CodecError;
use crate::{protocol::xml::CotLegacyCodec, router::Router};
use futures::StreamExt;
use std::io::ErrorKind;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    select,
};
use tokio_util::codec::Framed;

fn unexpected_eof_is_none<V>(res: Option<Result<V, CodecError>>) -> Option<Result<V, CodecError>> {
    match res {
        // because of https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
        Some(Err(CodecError::Io(e))) if e.kind() == ErrorKind::UnexpectedEof => None,
        res => res,
    }
}

pub struct CotClientConnection<T> {
    io_stream: T,
    connection_id: String,
    router: Router,
}

impl<T> CotClientConnection<T> {
    pub fn new(io_stream: T, connection_id: String, router: Router) -> Self {
        Self {
            io_stream,
            connection_id,
            router,
        }
    }
}

struct Defer<F>
where
    F: FnMut() -> (),
{
    f: F,
}

impl<F> Drop for Defer<F>
where
    F: FnMut() -> (),
{
    fn drop(&mut self) {
        (self.f)()
    }
}

fn defer<F: FnMut() -> ()>(f: F) -> Defer<F> {
    Defer { f }
}

impl<T: AsyncRead + AsyncWrite> CotClientConnection<T> {
    pub async fn conn_loop(self) -> anyhow::Result<()> {
        let router = self.router.clone();
        let connection_id = self.connection_id.clone();
        let _deref = defer(move || {
            router.connection_dropped(&connection_id);
        });

        let frames = Framed::new(self.io_stream, CotLegacyCodec::new(4 * 1024));
        let (mut frame_writer, mut frame_stream) = frames.split();

        loop {
            select! {
                maybe_frame_res = frame_stream.next() => {
                    if let Some(frame_res) = unexpected_eof_is_none(maybe_frame_res) {
                        let message = frame_res?;
                        self.router.cot_packet_received(&self.connection_id, message)?;
                    } else {
                        break
                    }

                }
            }
        }
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
        let client_conn =
            CotClientConnection::new(UnexpectedEOFReader, "test conn".into(), Router::new(1));
        let res = client_conn.conn_loop().await;
        assert!(res.is_ok())
    }
}
