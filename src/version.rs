//! HTTP version
//!
//! This module contains a definition of the `Version` type. The `Version`
//! type is intended to be accessed through the root of the crate
//! (`http::Version`) rather than this module.
//!
//! The `Version` type contains constants that represent the various versions
//! of the HTTP protocol.
//!
//! # Examples
//!
//! ```
//! use http::Version;
//!
//! let http11 = Version::HTTP_11;
//! let http2 = Version::HTTP_2;
//! assert!(http11 != http2);
//!
//! println!("{:?}", http2);
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents a version of the HTTP spec.
#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash, Deserialize, Serialize)]
pub struct Version(Http);

impl Version {
    /// `HTTP/0.9`
    pub const HTTP_09: Version = Version(Http::Http09);

    /// `HTTP/1.0`
    pub const HTTP_10: Version = Version(Http::Http10);

    /// `HTTP/1.1`
    pub const HTTP_11: Version = Version(Http::Http11);

    /// `HTTP/2.0`
    pub const HTTP_2: Version = Version(Http::H2);

    /// `HTTP/3.0`
    pub const HTTP_3: Version = Version(Http::H3);
}

impl From<http::Version> for Version {
    fn from(version: http::Version) -> Self {
        match version {
            http::Version::HTTP_09 => Version::HTTP_09,
            http::Version::HTTP_10 => Version::HTTP_10,
            http::Version::HTTP_11 => Version::HTTP_11,
            http::Version::HTTP_2 => Version::HTTP_2,
            http::Version::HTTP_3 => Version::HTTP_3,
            _ => unimplemented!(),
        }
    }
}

impl From<Version> for http::Version {
    fn from(version: Version) -> Self {
        match version {
            Version::HTTP_09 => http::Version::HTTP_09,
            Version::HTTP_10 => http::Version::HTTP_10,
            Version::HTTP_11 => http::Version::HTTP_11,
            Version::HTTP_2 => http::Version::HTTP_2,
            Version::HTTP_3 => http::Version::HTTP_3,
            _ => unimplemented!(),
        }
    }
}

#[derive(PartialEq, PartialOrd, Copy, Clone, Eq, Ord, Hash, Deserialize, Serialize)]
enum Http {
    Http09,
    Http10,
    Http11,
    H2,
    H3,
    __NonExhaustive,
}

impl Default for Version {
    #[inline]
    fn default() -> Version {
        Version::HTTP_11
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Http::*;

        f.write_str(match self.0 {
            Http09 => "HTTP/0.9",
            Http10 => "HTTP/1.0",
            Http11 => "HTTP/1.1",
            H2 => "HTTP/2.0",
            H3 => "HTTP/3.0",
            __NonExhaustive => unreachable!(),
        })
    }
}
