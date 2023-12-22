use tokio_util::bytes::BytesMut;
use tokio_util::codec::Decoder;

pub const COT_LEGACY_FRAME_MARKER: &[u8] = b"</event>";

pub struct PatternSplitDecoder<'a> {
    pattern: &'a [u8],
}

impl<'a> PatternSplitDecoder<'a> {
    pub fn new(pattern: &'a [u8]) -> Self {
        Self { pattern }
    }
}
impl Decoder for PatternSplitDecoder<'_> {
    type Item = Vec<u8>;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(if let Some(pos) = find_in(src, self.pattern) {
            let frame = src.split_to(pos + self.pattern.len());
            Some(frame.to_vec())
        } else {
            None
        })
    }
}

fn find_in(slice: &[u8], subslice: &[u8]) -> Option<usize> {
    slice
        .windows(subslice.len())
        .enumerate()
        .find(|&(_, window)| window == subslice)
        .map(|(pos, _)| pos)
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_util::bytes::{BufMut, BytesMut};

    #[tokio::test]
    async fn xml_decoder_test() -> anyhow::Result<()> {
        let data =
            b"<xml header1>\r\n<event>something something</event><xml header2>\r\n<event>something again";
        let mut buffer = BytesMut::from(data.as_slice());
        let mut decoder = PatternSplitDecoder::new(COT_LEGACY_FRAME_MARKER);

        let frame1 = decoder.decode(&mut buffer)?.expect("should be present");
        assert_eq!(
            "<xml header1>\r\n<event>something something</event>",
            String::from_utf8_lossy(&frame1)
        );

        buffer.put_slice(b"</event>".as_slice());

        let frame2 = decoder.decode(&mut buffer)?.expect("should be present");
        assert_eq!(
            "<xml header2>\r\n<event>something again</event>",
            String::from_utf8_lossy(&frame2)
        );

        let frame3 = decoder.decode(&mut buffer)?;
        assert_eq!(None, frame3);

        buffer.put_slice(b"<event>abc</event>");
        let frame4 = decoder.decode(&mut buffer)?.expect("should be present");
        assert_eq!("<event>abc</event>", String::from_utf8_lossy(&frame4));

        Ok(())
    }
}
