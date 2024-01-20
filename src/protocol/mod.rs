use std::io::Write;

pub mod xml;

/// main Cot message, for legacy protocol should be convertable to xml
/// for version 1 - to special Cot PROTO message (not avaialble yet

#[derive(Debug)]
pub enum Message {
    Xml(minidom::Element),
}

impl Message {
    pub fn as_xml<T: Write>(&self, writer: &mut T) -> anyhow::Result<()> {
        match self {
            Message::Xml(elem) => elem.write_to(writer)?,
        }
        Ok(())
    }
}
