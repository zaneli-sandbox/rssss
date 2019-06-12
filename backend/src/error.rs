use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use serde_derive::Serialize;
use std::collections::HashMap;
use xml::reader::Error as XMLReaderError;

pub struct InvalidRequestError<K, V>
where
    K: ToString,
    V: ToString,
{
    pub messages: Vec<(K, V)>,
}

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

impl<K, V> From<InvalidRequestError<K, V>> for Error<HashMap<&str, String>>
where
    K: ToString,
    V: ToString,
{
    fn from(error: InvalidRequestError<K, V>) -> Error<HashMap<&'static str, String>> {
        let messages: Vec<HashMap<&'static str, String>> = error
            .messages
            .into_iter()
            .map(|(key, value)| {
                [("key", key.to_string()), ("value", value.to_string())]
                    .iter()
                    .cloned()
                    .collect()
            })
            .collect();
        Error { messages: messages }
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
