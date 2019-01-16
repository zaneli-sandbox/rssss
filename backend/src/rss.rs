use bytes::buf::IntoBuf;
use error::Error;
use error::ErrorKind::InvalidRssError;
use scraper::Html;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
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
    let mut tag = (Option::default(), String::new());
    let mut title = String::new();
    let mut link = String::new();
    let mut description = String::new();
    let mut pub_date = Option::default();
    let mut rs: Vec<Rss> = Vec::new();

    let mut root = true;
    for elem in parser {
        match elem? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if root {
                    verify_rss(&name, &attributes)?;
                    root = false;
                }
                tag = (name.namespace, name.local_name);
            }
            XmlEvent::Characters(data) | XmlEvent::CData(data) => {
                let (namespace, local_name) = &tag;
                match (namespace.as_ref().map(|n| n.as_ref()), local_name.as_ref()) {
                    (_, "title") => title = data,
                    (_, "link") => link = data,
                    (_, "description") => description = pick_texts(data),
                    (Some("http://purl.org/rss/1.0/modules/content/"), "encoded") => {
                        if description.is_empty() {
                            description = pick_texts(data)
                        }
                    }
                    (_, "pubDate") => pub_date = Some(data),
                    _ => (),
                }
            }
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
                tag = (Option::default(), String::new());
            }
            _ => (),
        };
    }
    Ok(rs)
}

fn verify_rss(name: &OwnedName, attributes: &Vec<OwnedAttribute>) -> Result<(), Error> {
    if name.local_name != "rss" {
        return Err(Error::from(InvalidRssError(format!(
            "invalid root element name: {}",
            name.local_name
        ))));
    }
    let version = attributes
        .iter()
        .find(|a| a.name.to_string() == "version")
        .map(|a| a.value.as_ref());
    match version {
        Some("2.0") => Ok(()),
        Some(version) => Err(Error::from(InvalidRssError(format!(
            "unsupported rss version: {}",
            version
        )))),
        None => Err(Error::from(InvalidRssError(
            "missing version attribute".to_string(),
        ))),
    }
}

fn pick_texts(data: String) -> String {
    let document = Html::parse_document(data.as_ref());
    let mut texts = String::new();
    for text in document.root_element().text() {
        texts.push_str(text)
    }
    texts
}
