use crate::protocol::Message;
use minidom::Element;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub const COT_LEGACY_FRAME_MARKER: &[u8] = b"</event>";

pub struct CotCodec {
    buff: Vec<u8>,
}

impl CotCodec {
    pub fn new(buf_size: usize) -> Self {
        Self {
            buff: Vec::with_capacity(buf_size),
        }
    }
}

impl Decoder for CotCodec {
    type Item = Message;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(if let Some(pos) = find_in(src, COT_LEGACY_FRAME_MARKER) {
            let frame = src.split_to(pos + COT_LEGACY_FRAME_MARKER.len());
            let element = xml_parse(frame.as_ref())?;
            Some(Message::Xml(element))
        } else {
            None
        })
    }
}

impl Encoder<Message> for CotCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.buff.clear();
        item.as_xml(&mut self.buff)?;
        dst.extend_from_slice(&self.buff);
        Ok(())
    }
}

fn find_in(slice: &[u8], subslice: &[u8]) -> Option<usize> {
    slice
        .windows(subslice.len())
        .enumerate()
        .find(|&(_, window)| window == subslice)
        .map(|(pos, _)| pos)
}

fn xml_parse(xml: &[u8]) -> minidom::Result<Element> {
    // event xml element comes without ns, treat it as empty
    Element::from_reader_with_prefixes(xml, Some("".to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_util::bytes::{BufMut, BytesMut};

    #[tokio::test]
    async fn xml_decoder_test() -> anyhow::Result<()> {
        let data = b"<event>something something</event><event>something again";
        let mut buffer = BytesMut::from(data.as_slice());
        let mut result = Vec::with_capacity(2024);
        let mut decoder = CotCodec::new(2048);

        let frame1 = decoder.decode(&mut buffer)?.expect("should be present");
        frame1.as_xml(&mut result)?;
        assert_eq!(
            "<event>something something</event>",
            String::from_utf8_lossy(&result)
        );

        buffer.put_slice(b"</event>".as_slice());

        let frame2 = decoder.decode(&mut buffer)?.expect("should be present");
        result.clear();
        frame2.as_xml(&mut result)?;
        assert_eq!(
            "<event>something again</event>",
            String::from_utf8_lossy(&result)
        );

        let frame3 = decoder.decode(&mut buffer)?;
        assert!(frame3.is_none());

        buffer.put_slice(b"<event>abc</event>");
        let frame4 = decoder.decode(&mut buffer)?.expect("should be present");
        result.clear();
        frame4.as_xml(&mut result)?;
        assert_eq!("<event>abc</event>", String::from_utf8_lossy(&result));

        Ok(())
    }

    macro_rules! xml_test_message {
        ($name: literal) => {
            ($name, include_bytes!($name).as_slice())
        };
    }
    #[test]
    fn xml_message_parser() {
        let messages = [
            xml_test_message!("fixtures/first_event.xml"),
            xml_test_message!("fixtures/additional.xml"),
            xml_test_message!("fixtures/911_alert_start.xml"),
            xml_test_message!("fixtures/911_deactive.xml"),
            xml_test_message!("fixtures/contact_alert.xml"),
            xml_test_message!("fixtures/general_chat_message.xml"),
        ];

        for (name, content) in messages {
            let res = xml_parse(content);
            assert!(res.is_ok(), "Assertion failed for {name}, error: {res:#?}");
            println!("{:#?}", res.unwrap())
        }
    }
}
