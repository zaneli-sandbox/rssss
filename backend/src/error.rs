use actix_web::error::PayloadError;
use actix_web::ResponseError;
use awc::error::SendRequestError;
use serde_derive::Serialize;
use xml::reader::Error as XMLReaderError;

pub struct InvalidRssError {
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct Error<T> {
    messages: Vec<T>,
}

impl From<XMLReaderError> for Error<String> {
    fn from(error: XMLReaderError) -> Error<String> {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<PayloadError> for Error<String> {
    fn from(error: PayloadError) -> Error<String> {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<SendRequestError> for Error<String> {
    fn from(error: SendRequestError) -> Error<String> {
        Error {
            messages: vec![error.to_string()],
        }
    }
}

impl From<InvalidRssError> for Error<String> {
    fn from(error: InvalidRssError) -> Error<String> {
        Error {
            messages: vec![error.message],
        }
    }
}

impl<T> From<Vec<Error<T>>> for Error<T> {
    fn from(errors: Vec<Error<T>>) -> Error<T> {
        let mut messages = Vec::new();
        for error in errors {
            for message in error.messages {
                messages.push(message);
            }
        }
        Error { messages: messages }
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let messages = self
            .messages
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{}", messages)
    }
}

impl<T: std::fmt::Debug + std::fmt::Display> ResponseError for Error<T> {}
