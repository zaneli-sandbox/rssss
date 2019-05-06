use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use serde_derive::Serialize;
use xml::reader::Error as XMLReaderError;

pub struct InvalidRssError {
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct Error {
    messages: Vec<String>,
}

impl From<XMLReaderError> for Error {
    fn from(error: XMLReaderError) -> Error {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<PayloadError> for Error {
    fn from(error: PayloadError) -> Error {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<SendRequestError> for Error {
    fn from(error: SendRequestError) -> Error {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<InvalidRssError> for Error {
    fn from(error: InvalidRssError) -> Error {
        Error {
            messages: vec![error.message],
        }
    }
}

impl From<Vec<Error>> for Error {
    fn from(errors: Vec<Error>) -> Error {
        let mut messages = Vec::new();
        for error in errors {
            for message in error.messages {
                messages.push(message);
            }
        }
        Error { messages: messages }
    }
}
