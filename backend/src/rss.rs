use bytes::buf::IntoBuf;
use error::Error;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize)]
pub struct Rss {
    title: String,
    description: String,
    link: String,
    pub_date: Option<String>,
}

pub fn parse_rss(buf: bytes::Bytes) -> Result<Vec<Rss>, Error> {
    let parser = EventReader::new(buf.into_buf());
    let mut tag = String::new();
    let mut title = String::new();
    let mut link = String::new();
    let mut description = String::new();
    let mut pub_date = Option::default();
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
                "pubDate" => pub_date = Some(text),
                _ => (),
            },
            XmlEvent::EndElement { name } => {
                if name.to_string() == "item" {
                    rs.push(Rss {
                        title: title.clone(),
                        link: link.clone(),
                        description: description.clone(),
                        pub_date: pub_date.clone(),
                    });
                    title = String::new();
                    link = String::new();
                    description = String::new();
                    pub_date = Option::default();
                }
                tag = String::new();
            }
            _ => (),
        };
    }
    Ok(rs)
}
