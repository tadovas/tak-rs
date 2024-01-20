use crate::protocol;
use futures::{pin_mut, StreamExt};
use std::io::ErrorKind;
use tokio::io::AsyncRead;
use tokio_util::codec::FramedRead;
use tracing::info;

fn unexpected_eof_is_none<V>(
    res: Option<Result<V, std::io::Error>>,
) -> Option<Result<V, std::io::Error>> {
    match res {
        // because of https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
        Some(Err(e)) if e.kind() == ErrorKind::UnexpectedEof => None,
        res => res,
    }
}

pub(super) async fn client_loop<S: AsyncRead>(stream: S) -> anyhow::Result<()> {
    let frames = FramedRead::new(
        stream,
        protocol::xml::PatternSplitDecoder::new(protocol::xml::COT_LEGACY_FRAME_MARKER),
    );
    pin_mut!(frames);

    while let Some(res) = unexpected_eof_is_none(frames.next().await) {
        let xml_packet = res?;
        info!("XML Packet:");
        info!("{}", String::from_utf8_lossy(&xml_packet));
        info!("XML END");
    }
    info!("Disconnected");
    Ok(())
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
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::Error::new(ErrorKind::UnexpectedEof, "test error")))
        }
    }

    #[tokio::test]
    async fn test_client_disconnection_without_err() {
        //FIXME - need a guard against infinite loop
        let res = client_loop(UnexpectedEOFReader).await;
        assert!(res.is_ok())
    }
}
