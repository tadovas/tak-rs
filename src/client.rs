use crate::protocol;
use futures::{pin_mut, StreamExt};
use tokio::io::AsyncRead;
use tokio_util::codec::FramedRead;
use tracing::info;

pub async fn client_loop<S: AsyncRead>(stream: S) -> anyhow::Result<()> {
    let frames = FramedRead::new(
        stream,
        protocol::xml::PatternSplitDecoder::new(protocol::xml::COT_LEGACY_FRAME_MARKER),
    );
    pin_mut!(frames);
    while let Some(res) = frames.next().await {
        let xml_packet = res?;
        info!("XML Packet:");
        info!("{}", String::from_utf8_lossy(&xml_packet));
        info!("XML END");
    }
    info!("End of client frames");
    Ok(())
}
