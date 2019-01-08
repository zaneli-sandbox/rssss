use bytes::buf::IntoBuf;
use error::Error;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize)]
pub struct Rss {
    title: String,
    description: String,
    link: String,
}

pub fn parse_rss(buf: bytes::Bytes) -> Result<Vec<Rss>, Error> {
    let parser = EventReader::new(buf.into_buf());
    let mut tag = String::new();
    let mut title = String::new();
    let mut link = String::new();
    let mut description = String::new();
    let mut rs: Vec<Rss> = Vec::new();
    for elem in parser {
        match elem? {
            XmlEvent::StartElement { name, .. } => {
                tag = name.to_string();
            }
            XmlEvent::Characters(text) => match tag.as_ref() {
                "title" => title = text,
                "link" => link = text,
                "description" => description = text,
                _ => (),
            },
            XmlEvent::EndElement { name } => {
                if name.to_string() == "item" {
                    rs.push(Rss {
                        title: title.clone(),
                        link: link.clone(),
                        description: description.clone(),
                    });
                    title = String::new();
                    link = String::new();
                    description = String::new();
                }
                tag = String::new();
            }
            _ => (),
        };
    }
    Ok(rs)
}
