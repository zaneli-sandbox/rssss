use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use xml::reader::Error as XMLReaderError;

#[derive(Serialize)]
pub struct ResponseError {
    message: String,
}

pub struct InvalidRssError {
    pub message: String,
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl From<Error> for ResponseError {
    fn from(e: Error) -> ResponseError {
        ResponseError { message: e.message }
    }
}

impl From<XMLReaderError> for Error {
    fn from(error: XMLReaderError) -> Error {
        Error {
            message: error.to_string(),
        }
    }
}

impl From<PayloadError> for Error {
    fn from(error: PayloadError) -> Error {
        Error {
            message: error.to_string(),
        }
    }
}

impl From<SendRequestError> for Error {
    fn from(error: SendRequestError) -> Error {
        Error {
            message: error.to_string(),
        }
    }
}

impl From<InvalidRssError> for Error {
    fn from(error: InvalidRssError) -> Error {
        Error {
            message: error.message,
        }
    }
}

impl From<Vec<Error>> for Error {
    fn from(errors: Vec<Error>) -> Error {
        let mut messages = String::new();
        for error in errors {
            messages.push_str(error.message.as_ref());
        }
        Error { message: messages }
    }
}
