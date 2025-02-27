use serde::Deserialize;
use serde::Serialize;
use std::error::Error as StdError;
use std::fmt;
use std::io;

use crate::SerializableResponse;
use crate::{StatusCode, Url};

/// A `Result` alias where the `Err` case is `nightfly::Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
pub struct ResponseResult {
    result: std::result::Result<SerializableResponse, Error>,
}

/// The Errors that may occur when processing a `Request`.
///
/// Note: Errors may include the full URL used to make the `Request`. If the URL
/// contains sensitive information (e.g. an API key as a query parameter), be
/// sure to remove it ([`without_url`](Error::without_url))
#[derive(Serialize, Deserialize, Clone)]
pub struct Error {
    inner: Box<Inner>,
}

pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Serialize, Deserialize)]
struct Inner {
    kind: Kind,
    #[serde(skip)]
    source: Option<BoxError>,
    url: Option<Url>,
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        Inner {
            kind: self.kind.clone(),
            source: None,
            url: self.url.clone(),
        }
    }
}

impl Error {
    pub(crate) fn new<E>(kind: Kind, source: Option<E>) -> Error
    where
        E: Into<BoxError>,
    {
        Error {
            inner: Box::new(Inner {
                kind,
                source: source.map(Into::into),
                url: None,
            }),
        }
    }

    /// Returns a possible URL related to this error.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn run() {
    /// // displays last stop of a redirect loop
    /// let response = nightfly::get("http://site.with.redirect.loop");
    /// if let Err(e) = response {
    ///     if e.is_redirect() {
    ///         if let Some(final_stop) = e.url() {
    ///             println!("redirect loop at {}", final_stop);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn url(&self) -> Option<&Url> {
        self.inner.url.as_ref()
    }

    /// Returns a mutable reference to the URL related to this error
    ///
    /// This is useful if you need to remove sensitive information from the URL
    /// (e.g. an API key in the query), but do not want to remove the URL
    /// entirely.
    pub fn url_mut(&mut self) -> Option<&mut Url> {
        self.inner.url.as_mut()
    }

    /// Add a url related to this error (overwriting any existing)
    pub fn with_url(mut self, url: Url) -> Self {
        self.inner.url = Some(url);
        self
    }

    /// Strip the related url from this error (if, for example, it contains
    /// sensitive information)
    pub fn without_url(mut self) -> Self {
        self.inner.url = None;
        self
    }

    /// Returns true if the error is from a type Builder.
    pub fn is_builder(&self) -> bool {
        matches!(self.inner.kind, Kind::Builder)
    }

    /// Returns true if the error is from a `RedirectPolicy`.
    pub fn is_redirect(&self) -> bool {
        matches!(self.inner.kind, Kind::Redirect)
    }

    /// Returns true if the error is from `Response::error_for_status`.
    pub fn is_status(&self) -> bool {
        matches!(self.inner.kind, Kind::Status(_))
    }

    /// Returns true if the error is related to a timeout.
    pub fn is_timeout(&self) -> bool {
        let mut source = self.source();

        while let Some(err) = source {
            if err.is::<TimedOut>() {
                return true;
            }
            source = err.source();
        }

        false
    }

    /// Returns true if the error is related to the request
    pub fn is_request(&self) -> bool {
        matches!(self.inner.kind, Kind::Request)
    }

    /// Returns true if the error is related to the request or response body
    // pub fn is_body(&self) -> bool {
    //     matches!(self.inner.kind, Kind::Body)
    // }

    /// Returns true if the error is related to the serialisation of the body
    pub fn is_serialization(&self) -> bool {
        matches!(self.inner.kind, Kind::Serialization)
    }

    /// Returns true if the error is related to decoding the response's body
    pub fn is_decode(&self) -> bool {
        matches!(self.inner.kind, Kind::Decode)
    }

    /// Returns the status code, if the error was generated from a response.
    pub fn status(&self) -> Option<StatusCode> {
        match self.inner.kind {
            Kind::Status(code) => Some(StatusCode::from_u16(code).unwrap()),
            _ => None,
        }
    }

    // private

    #[allow(unused)]
    pub(crate) fn into_io(self) -> io::Error {
        io::Error::new(io::ErrorKind::Other, self)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("nightfly::Error");

        builder.field("kind", &self.inner.kind);

        if let Some(ref url) = self.inner.url {
            builder.field("url", url);
        }
        if let Some(ref source) = self.inner.source {
            builder.field("source", source);
        }

        builder.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner.kind {
            Kind::Builder => f.write_str("builder error")?,
            Kind::Request => f.write_str("error sending request")?,
            Kind::Body => f.write_str("request or response body error")?,
            Kind::Decode => f.write_str("error decoding response body")?,
            Kind::Redirect => f.write_str("error following redirect")?,
            Kind::Serialization => f.write_str("error while serialising body")?,
            // Kind::Upgrade => f.write_str("error upgrading connection")?,
            Kind::Status(ref code) => {
                let status = StatusCode::from_u16(*code).unwrap();
                let prefix = if status.is_client_error() {
                    "HTTP status client error"
                } else {
                    debug_assert!(status.is_server_error());
                    "HTTP status server error"
                };
                write!(f, "{} ({})", prefix, code)?;
            }
        };

        if let Some(url) = &self.inner.url {
            write!(f, " for url ({})", url.as_str())?;
        }

        if let Some(e) = &self.inner.source {
            write!(f, ": {}", e)?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum Kind {
    Builder,
    Request,
    Redirect,
    Status(u16),
    Body,
    Decode,
    Serialization,
    // Upgrade,
}

// constructors

pub(crate) fn builder<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Builder, Some(e))
}

pub(crate) fn serialization<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Serialization, Some(e))
}

// pub(crate) fn body<E: Into<BoxError>>(e: E) -> Error {
//     Error::new(Kind::Body, Some(e))
// }

pub(crate) fn decode<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Decode, Some(e))
}

pub(crate) fn request<E: Into<BoxError>>(e: E) -> Error {
    Error::new(Kind::Request, Some(e))
}

pub(crate) fn timeout(url: Url) -> Error {
    Error::new(Kind::Request, Some(TimedOut)).with_url(url)
}

pub(crate) fn redirect<E: Into<BoxError>>(e: E, url: Url) -> Error {
    Error::new(Kind::Redirect, Some(e)).with_url(url)
}

pub(crate) fn status_code(url: Url, status: StatusCode) -> Error {
    Error::new(Kind::Status(status.as_u16()), None::<Error>).with_url(url)
}

pub(crate) fn url_bad_scheme(url: Url) -> Error {
    Error::new(Kind::Builder, Some(BadScheme)).with_url(url)
}

// pub(crate) fn upgrade<E: Into<BoxError>>(e: E) -> Error {
//     Error::new(Kind::Upgrade, Some(e))
// }

// io::Error helpers

#[allow(unused)]
pub(crate) fn into_io(e: Error) -> io::Error {
    e.into_io()
}

#[allow(unused)]
pub(crate) fn decode_io(e: io::Error) -> Error {
    if e.get_ref().map(|r| r.is::<Error>()).unwrap_or(false) {
        *e.into_inner()
            .expect("io::Error::get_ref was Some(_)")
            .downcast::<Error>()
            .expect("StdError::is() was true")
    } else {
        decode(e)
    }
}

// internal Error "sources"

#[derive(Debug)]
pub(crate) struct TimedOut;

impl fmt::Display for TimedOut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("operation timed out")
    }
}

impl StdError for TimedOut {}

#[derive(Debug)]
pub(crate) struct BadScheme;

impl fmt::Display for BadScheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("URL scheme is not allowed")
    }
}

impl StdError for BadScheme {}

// #[cfg(test)]
mod tests {
    use super::*;

    #[lunatic::test]
    fn mem_size_of() {
        use std::mem::size_of;
        assert_eq!(size_of::<Error>(), size_of::<usize>());
    }

    #[lunatic::test]
    fn from_unknown_io_error() {
        let orig = io::Error::new(io::ErrorKind::Other, "orly");
        let err = super::decode_io(orig);
        match err.inner.kind {
            Kind::Decode => (),
            _ => panic!("{:?}", err),
        }
    }

    #[lunatic::test]
    fn is_timeout() {
        let err = super::timeout(Url::parse("http://localhost:3000/api").unwrap());
        assert!(err.is_timeout());

        let io = io::Error::new(io::ErrorKind::Other, err);
        let nested = super::request(io);
        assert!(nested.is_timeout());
    }
}
