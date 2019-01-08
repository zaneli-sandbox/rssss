use actix_web::client::SendRequestError;
use actix_web::error::PayloadError;
use failure::{Backtrace, Context, Fail};
use std::fmt;
use std::fmt::Display;
use xml::reader::Error as XMLReaderError;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "xml reader error")]
    XMLReader,
    #[fail(display = "actix payload error")]
    PayloadError,
    #[fail(display = "actix client send request error")]
    SendRequestError,
}

impl From<XMLReaderError> for Error {
    fn from(error: XMLReaderError) -> Error {
        Error {
            inner: error.context(ErrorKind::XMLReader),
        }
    }
}

impl From<PayloadError> for Error {
    fn from(error: PayloadError) -> Error {
        Error {
            inner: error.context(ErrorKind::PayloadError),
        }
    }
}

impl From<SendRequestError> for Error {
    fn from(error: SendRequestError) -> Error {
        Error {
            inner: error.context(ErrorKind::SendRequestError),
        }
    }
}

/* ----------- failure boilerplate ----------- */

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn new(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}
