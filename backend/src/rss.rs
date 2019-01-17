use bytes::buf::IntoBuf;
use error::Error;
use error::ErrorKind::InvalidRssError;
use scraper::Html;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize, Clone)]
pub struct Rss {
    title: String,
    description: String,
    link: String,
    pub_date: Option<String>,
}

impl Rss {
    fn new(title: String, description: String, link: String, pub_date: Option<String>) -> Rss {
        Rss {
            title: Rss::trim(title),
            description: Rss::trim(Rss::pick_texts(description)),
            link: link,
            pub_date: pub_date,
        }
    }
    fn trim(s: String) -> String {
        s.trim_start().trim_end().to_string()
    }
    fn pick_texts(data: String) -> String {
        let document = Html::parse_document(data.as_ref());
        let mut texts = String::new();
        for text in document.root_element().text() {
            texts.push_str(text)
        }
        texts
    }
}

pub fn parse_rss(buf: bytes::Bytes) -> Result<Vec<Rss>, Error> {
    parse(&buf, &mut RssV20::new()).or(parse(&buf, &mut Atom::new()))
}

fn parse(buf: &bytes::Bytes, parser: &mut RssParser) -> Result<Vec<Rss>, Error> {
    let reader = EventReader::new(buf.into_buf());

    let mut root = true;
    for elem in reader {
        match elem? {
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                parser.parse_start_element(name, attributes, namespace);
                if root {
                    parser.verify_rss()?;
                    root = false;
                }
            }
            XmlEvent::Characters(data) | XmlEvent::CData(data) => {
                parser.parse_content(data);
            }
            XmlEvent::EndElement { name } => {
                parser.parse_end_element(name);
            }
            _ => (),
        };
    }
    Ok(parser.get_results())
}

trait RssParser {
    fn parse_start_element(&mut self, OwnedName, Vec<OwnedAttribute>, Namespace);
    fn parse_content(&mut self, String);
    fn parse_end_element(&mut self, OwnedName);
    fn verify_rss(&self) -> Result<(), Error>;
    fn get_results(&self) -> Vec<Rss>;
}

struct RssV20 {
    results: Vec<Rss>,
    element: (OwnedName, Vec<OwnedAttribute>, Namespace),
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

impl RssV20 {
    fn new() -> RssV20 {
        RssV20 {
            results: Vec::new(),
            element: (
                OwnedName::local(String::new()),
                Vec::default(),
                Namespace::empty(),
            ),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            pub_date: Option::default(),
        }
    }
}

impl RssParser for RssV20 {
    fn parse_start_element(&mut self, name: OwnedName, attrs: Vec<OwnedAttribute>, ns: Namespace) {
        self.element = (name, attrs, ns);
    }
    fn parse_content(&mut self, data: String) {
        let (name, _, _) = &self.element;
        match (
            name.namespace.as_ref().map(|n| n.as_ref()),
            name.local_name.as_ref(),
        ) {
            (_, "title") => self.title = data,
            (_, "link") => self.link = data,
            (_, "description") => self.description = data,
            (Some("http://purl.org/rss/1.0/modules/content/"), "encoded") => {
                if self.description.is_empty() {
                    self.description = data;
                }
            }
            (_, "pubDate") => self.pub_date = Some(data),
            _ => (),
        }
    }
    fn parse_end_element(&mut self, name: OwnedName) {
        if name.to_string() == "item" {
            let rss = Rss::new(
                self.title.clone(),
                self.description.clone(),
                self.link.clone(),
                self.pub_date.clone(),
            );
            self.results.push(rss);

            self.title = String::new();
            self.link = String::new();
            self.description = String::new();
            self.pub_date = Option::default();
        }
        self.element = (
            OwnedName::local(String::new()),
            Vec::default(),
            Namespace::empty(),
        );
    }
    fn verify_rss(&self) -> Result<(), Error> {
        let (name, attrs, _) = &self.element;
        if name.local_name != "rss" {
            return Err(Error::from(InvalidRssError(format!(
                "invalid root element name: {}",
                name.local_name
            ))));
        }
        let version = attrs
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
    fn get_results(&self) -> Vec<Rss> {
        self.results.clone()
    }
}

struct Atom {
    results: Vec<Rss>,
    element: (OwnedName, Vec<OwnedAttribute>, Namespace),
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

impl Atom {
    fn new() -> Atom {
        Atom {
            results: Vec::new(),
            element: (
                OwnedName::local(String::new()),
                Vec::default(),
                Namespace::empty(),
            ),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            pub_date: Option::default(),
        }
    }
}

impl RssParser for Atom {
    fn parse_start_element(&mut self, name: OwnedName, attrs: Vec<OwnedAttribute>, ns: Namespace) {
        if name.namespace == Some("http://www.w3.org/2005/Atom".to_string())
            && name.local_name == "link"
            && attrs
                .iter()
                .find(|a| {
                    a.name.to_string() == "rel" && a.value != "self" && a.value != "alternate"
                }).is_none()
        {
            attrs
                .iter()
                .find(|a| a.name.to_string() == "href")
                .map(|a| self.link = a.value.clone());
        }
        self.element = (name, attrs, ns);
    }
    fn parse_content(&mut self, data: String) {
        let (name, _, _) = &self.element;
        match (
            name.namespace.as_ref().map(|n| n.as_ref()),
            name.local_name.as_ref(),
        ) {
            (Some("http://www.w3.org/2005/Atom"), "title") => self.title = data,
            (Some("http://www.w3.org/2005/Atom"), "content") => self.description = data,
            (Some("http://www.w3.org/2005/Atom"), "published") => self.pub_date = Some(data),
            (Some("http://www.w3.org/2005/Atom"), "updated") => {
                if self.pub_date.is_none() {
                    self.pub_date = Some(data);
                }
            }
            _ => (),
        }
    }
    fn parse_end_element(&mut self, name: OwnedName) {
        if name.namespace == Some("http://www.w3.org/2005/Atom".to_string())
            && name.local_name == "entry"
        {
            let rss = Rss::new(
                self.title.clone(),
                self.description.clone(),
                self.link.clone(),
                self.pub_date.clone(),
            );
            self.results.push(rss);

            self.title = String::new();
            self.link = String::new();
            self.description = String::new();
            self.pub_date = Option::default();
        }
        self.element = (
            OwnedName::local(String::new()),
            Vec::default(),
            Namespace::empty(),
        );
    }
    fn verify_rss(&self) -> Result<(), Error> {
        let (name, _, _) = &self.element;
        if name.local_name != "feed"
            || name.namespace != Some("http://www.w3.org/2005/Atom".to_string())
        {
            return Err(Error::from(InvalidRssError(format!(
                "invalid root element: {}",
                name
            ))));
        }
        Ok(())
    }
    fn get_results(&self) -> Vec<Rss> {
        self.results.clone()
    }
}
