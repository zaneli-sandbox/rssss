use crate::error::{Error, InvalidRssError};
use actix_web::web::Bytes;
use log::warn;
use scraper::Html;
use serde_derive::Serialize;
use std::collections::VecDeque;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize, Clone)]
pub struct Rss {
    title: String,
    description: String,
    link: String,
    pub_date: Option<String>,
}

impl Rss {
    const RDF_NS: &'static str = "http://purl.org/rss/1.0/";
    const RDF_SYNTAX_NS: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    const ELEMENTS_NS: &'static str = "http://purl.org/dc/elements/1.1/";
    const CONTENT_NS: &'static str = "http://purl.org/rss/1.0/modules/content/";

    const ATOM_NS: &'static str = "http://www.w3.org/2005/Atom";
    const MEDIA_NS: &'static str = "http://search.yahoo.com/mrss/";

    const DESCRIPTION_LIMIT: usize = 500;

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
            if texts.len() > Rss::DESCRIPTION_LIMIT {
                break;
            }
            let limit = Rss::DESCRIPTION_LIMIT - texts.len();
            if text.len() > limit {
                let mut end: usize = 0;
                text.chars()
                    .into_iter()
                    .take(limit)
                    .for_each(|x| end += x.len_utf8());
                texts.push_str(&text[..end]);
                texts.push_str("...")
            } else {
                texts.push_str(text)
            }
        }
        texts
    }
}

pub fn parse_rss(buf: Bytes) -> Result<Vec<Rss>, Error<String>> {
    let mut errors = Vec::new();
    let result = parse(&buf, &mut RssV20::new());
    if result.is_ok() {
        return result;
    }
    let _ = result.map_err(|e| errors.push(e));

    let result = parse(&buf, &mut Atom::new());
    if result.is_ok() {
        return result;
    }
    let _ = result.map_err(|e| errors.push(e));

    let result = parse(&buf, &mut RssV10::new());
    if result.is_ok() {
        return result;
    }
    let _ = result.map_err(|e| errors.push(e));

    return Err(errors.into());
}

fn parse(buf: &Bytes, parser: &mut dyn RssParser) -> Result<Vec<Rss>, Error<String>> {
    let reader = EventReader::new(buf.as_ref());

    let mut root = true;
    for elem in reader {
        match elem? {
            XmlEvent::StartDocument { encoding, .. } => {
                if encoding.to_uppercase() != "UTF-8" {
                    return Err(InvalidRssError {
                        message: format!("[{}] unsupported encoding: {}", parser.name(), encoding),
                    }
                    .into());
                }
            }
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                parser.parse_start_element(name, attributes);
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
    fn name(&self) -> &str;
    fn parse_start_element(&mut self, _: OwnedName, _: Vec<OwnedAttribute>);
    fn parse_content(&mut self, _: String);
    fn parse_end_element(&mut self, _: OwnedName);
    fn verify_rss(&self) -> Result<(), Error<String>>;
    fn get_results(&self) -> Vec<Rss>;
}

struct RssV20 {
    results: Vec<Rss>,
    elements: VecDeque<(OwnedName, Vec<OwnedAttribute>)>,
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

impl RssV20 {
    fn new() -> RssV20 {
        RssV20 {
            results: Vec::new(),
            elements: VecDeque::default(),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            pub_date: Option::default(),
        }
    }
    fn is_item(name: &OwnedName) -> bool {
        name.to_string() == "item"
    }
}

impl RssParser for RssV20 {
    fn name(&self) -> &str {
        return "RSS V2";
    }
    fn parse_start_element(&mut self, name: OwnedName, attrs: Vec<OwnedAttribute>) {
        self.elements.push_front((name, attrs));
    }
    fn parse_content(&mut self, data: String) {
        if &self.elements.len() < &2 {
            return;
        }
        let (parent, _) = &self.elements[1];
        if !RssV20::is_item(parent) {
            return;
        }
        let (name, _) = &self.elements[0];
        match (name.namespace_ref(), name.local_name.as_str()) {
            (_, "title") => self.title = data,
            (_, "link") => self.link = data,
            (_, "description") => self.description = data,
            (Some(Rss::CONTENT_NS), "encoded") => {
                if self.description.is_empty() {
                    self.description = data;
                }
            }
            (_, "pubDate") => self.pub_date = Some(data),
            _ => (),
        }
    }
    fn parse_end_element(&mut self, name: OwnedName) {
        if RssV20::is_item(&name) {
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
        self.elements.pop_front();
    }
    fn verify_rss(&self) -> Result<(), Error<String>> {
        let (name, attrs) = &self.elements[0];
        if name.local_name != "rss" {
            return Err(InvalidRssError {
                message: format!(
                    "[{}] invalid root element: {}",
                    self.name(),
                    name.local_name
                ),
            }
            .into());
        }
        let version = attrs
            .iter()
            .find(|a| a.name.to_string() == "version")
            .map(|a| a.value.as_ref());
        match version {
            Some("2.0") => Ok(()),
            Some(version) => {
                warn!("unsupported RSS version: {}", version);
                Err(InvalidRssError {
                    message: format!("[{}] unsupported RSS version: {}", self.name(), version),
                }
                .into())
            }
            None => Err(InvalidRssError {
                message: format!("[{}] undefined RSS version", self.name()),
            }
            .into()),
        }
    }
    fn get_results(&self) -> Vec<Rss> {
        self.results.clone()
    }
}

struct Atom {
    results: Vec<Rss>,
    elements: VecDeque<(OwnedName, Vec<OwnedAttribute>)>,
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

impl Atom {
    fn new() -> Atom {
        Atom {
            results: Vec::new(),
            elements: VecDeque::default(),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            pub_date: Option::default(),
        }
    }

    // for YouTube RSS format
    fn is_media_description(&self) -> bool {
        if &self.elements.len() < &3 {
            return false;
        }
        let (name, _) = &self.elements[2];
        if !Atom::is_entry(name) {
            return false;
        }
        let (name, _) = &self.elements[1];
        if !Atom::is_media_ns(name, "group") {
            return false;
        }
        let (name, _) = &self.elements[0];
        return Atom::is_media_ns(name, "description");
    }

    fn is_entry(name: &OwnedName) -> bool {
        name.namespace_ref() == Some(Rss::ATOM_NS) && name.local_name == "entry"
    }
    fn is_media_ns(name: &OwnedName, local_name: &str) -> bool {
        name.namespace_ref() == Some(Rss::MEDIA_NS) && name.local_name == local_name
    }
}

impl RssParser for Atom {
    fn name(&self) -> &str {
        return "Atom";
    }
    fn parse_start_element(&mut self, name: OwnedName, attrs: Vec<OwnedAttribute>) {
        if name.namespace_ref() == Some(Rss::ATOM_NS)
            && name.local_name == "link"
            && attrs
                .iter()
                .find(|a| {
                    a.name.to_string() == "rel" && a.value != "self" && a.value != "alternate"
                })
                .is_none()
        {
            attrs
                .iter()
                .find(|a| a.name.to_string() == "href")
                .map(|a| self.link = a.value.clone());
        }
        self.elements.push_front((name, attrs));
    }
    fn parse_content(&mut self, data: String) {
        if self.is_media_description() && self.description.is_empty() {
            self.description = data;
            return;
        }
        if &self.elements.len() < &2 {
            return;
        }
        let (parent, _) = &self.elements[1];
        if Atom::is_entry(parent) {
            let (name, _) = &self.elements[0];
            match (name.namespace_ref(), name.local_name.as_str()) {
                (Some(Rss::ATOM_NS), "title") => self.title = data,
                (Some(Rss::ATOM_NS), "content") => self.description = data,
                (Some(Rss::ATOM_NS), "published") => self.pub_date = Some(data),
                (Some(Rss::ATOM_NS), "updated") => {
                    if self.pub_date.is_none() {
                        self.pub_date = Some(data);
                    }
                }
                _ => (),
            }
        }
    }
    fn parse_end_element(&mut self, name: OwnedName) {
        if Atom::is_entry(&name) {
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
        self.elements.pop_front();
    }
    fn verify_rss(&self) -> Result<(), Error<String>> {
        let (name, _) = &self.elements[0];
        if name.local_name != "feed" {
            return Err(InvalidRssError {
                message: format!(
                    "[{}] invalid root element: {}",
                    self.name(),
                    name.local_name
                ),
            }
            .into());
        }
        if name.namespace_ref() != Some(Rss::ATOM_NS) {
            return Err(InvalidRssError {
                message: format!(
                    "[{}] invalid root namespace: {:?}",
                    self.name(),
                    name.namespace_ref()
                ),
            }
            .into());
        }
        Ok(())
    }
    fn get_results(&self) -> Vec<Rss> {
        self.results.clone()
    }
}

struct RssV10 {
    results: Vec<Rss>,
    elements: VecDeque<(OwnedName, Vec<OwnedAttribute>)>,
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
}

impl RssV10 {
    fn new() -> RssV10 {
        RssV10 {
            results: Vec::new(),
            elements: VecDeque::default(),
            title: String::new(),
            link: String::new(),
            description: String::new(),
            pub_date: Option::default(),
        }
    }
    fn is_item(name: &OwnedName) -> bool {
        name.local_name.eq_ignore_ascii_case("item") && name.namespace_ref() == Some(Rss::RDF_NS)
    }
}

impl RssParser for RssV10 {
    fn name(&self) -> &str {
        return "RSS V1";
    }
    fn parse_start_element(&mut self, name: OwnedName, attrs: Vec<OwnedAttribute>) {
        self.elements.push_front((name, attrs));
    }
    fn parse_content(&mut self, data: String) {
        if &self.elements.len() < &2 {
            return;
        }
        let (parent, _) = &self.elements[1];
        if !RssV10::is_item(parent) {
            return;
        }
        let (name, _) = &self.elements[0];
        match (name.namespace_ref(), name.local_name.as_str()) {
            (Some(Rss::RDF_NS), "title") => self.title = data,
            (Some(Rss::RDF_NS), "link") => self.link = data,
            (Some(Rss::RDF_NS), "description") => self.description = data,
            (Some(Rss::CONTENT_NS), "encoded") => {
                if self.description.is_empty() {
                    self.description = data;
                }
            }
            (Some(Rss::ELEMENTS_NS), "date") => self.pub_date = Some(data),
            _ => (),
        }
    }
    fn parse_end_element(&mut self, name: OwnedName) {
        if RssV10::is_item(&name) {
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
        self.elements.pop_front();
    }
    fn verify_rss(&self) -> Result<(), Error<String>> {
        let (name, _) = &self.elements[0];
        if !name.local_name.eq_ignore_ascii_case("rdf") {
            return Err(InvalidRssError {
                message: format!(
                    "[{}] invalid root element: {}",
                    self.name(),
                    name.local_name
                ),
            }
            .into());
        }
        if name.namespace_ref() != Some(Rss::RDF_SYNTAX_NS) {
            return Err(InvalidRssError {
                message: format!(
                    "[{}] invalid root namespace: {:?}",
                    self.name(),
                    name.namespace_ref()
                ),
            }
            .into());
        }
        Ok(())
    }
    fn get_results(&self) -> Vec<Rss> {
        self.results.clone()
    }
}
