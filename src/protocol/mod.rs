use std::io::Write;

pub mod xml;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("xml parse: {0}")]
    XmlParse(minidom::Error),
    #[error("xml render: {0}")]
    XmlRender(minidom::Error),
}

/// main Cot message, for legacy protocol should be convertable to xml
/// for version 1 - to special Cot PROTO message (not avaialble yet

#[derive(Debug)]
pub enum Message {
    Xml(minidom::Element),
}

impl Message {
    pub fn as_xml<T: Write>(&self, writer: &mut T) -> Result<(), CodecError> {
        match self {
            Message::Xml(elem) => elem.write_to(writer).map_err(CodecError::XmlRender)?,
        }
        Ok(())
    }

    pub fn from_raw_xml(input: &str) -> Result<Message, CodecError> {
        minidom::Element::from_reader_with_prefixes(input.as_bytes(), Some("".to_string()))
            .map(Self::Xml)
            .map_err(CodecError::XmlParse)
    }
}
