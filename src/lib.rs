#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// #![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/nightfly/0.1.4")]

//! # nightfly
//!
//! The `nightfly` crate provides a convenient, higher-level HTTP
//! [`Client`][client].
//!
//! It handles many of the things that most people just expect an HTTP client
//! to do for them.
//!
//! - Reusable, cloneable and serialisable Clients
//! - Plain bodies, [JSON](#json), [urlencoded](#forms)
//! - Customizable [redirect policy](#redirect-policies)
//! - Uses vm-native [TLS]
//! - Cookies
//!
//! Additional learning resources include:
//!
//! - [Nightfly Repository Examples](https://github.com/SquattingSocrates/nightfly/tree/master/examples)
//!
//! ## Making a GET request
//!
//! For a single request, you can use the [`get`][get] shortcut method.
//!
//! ```rust
//! # fn run() -> Result<(), nightfly::Error> {
//! let body = nightfly::get("https://www.rust-lang.org").text();
//!
//! println!("body = {:?}", body);
//! # Ok(())
//! # }
//! ```
//!
//! **NOTE**: If you plan to perform multiple requests, it is best to create a
//! [`Client`][client] and reuse it, taking advantage of keep-alive connection
//! pooling.
//!
//! ## Making POST requests (or setting request bodies)
//!
//! There are several ways you can set the body of a request. The basic one is
//! by using the `body()` method of a [`RequestBuilder`][builder]. This lets you set the
//! exact raw bytes of what the body should be. It accepts various types,
//! including `String` and `Vec<u8>`. If you wish to pass a custom
//! type, you can use the `nightfly::Body` constructors.
//!
//! ```rust
//! # use nightfly::Error;
//! #
//! # fn run() -> Result<(), Error> {
//! let client = nightfly::Client::new();
//! let res = client.post("http://httpbin.org/post")
//!     .body("the exact body that is sent")
//!     .send();
//! # Ok(())
//! # }
//! ```
//!
//! ### Forms
//!
//! It's very common to want to send form data in a request body. This can be
//! done with any type that can be serialized into form data.
//!
//! This can be an array of tuples, or a `HashMap`, or a custom type that
//! implements [`Serialize`][serde].
//!
//! ```rust
//! # use nightfly::Error;
//! #
//! # fn run() -> Result<(), Error> {
//! // This will POST a body of `foo=bar&baz=quux`
//! let params = [("foo", "bar"), ("baz", "quux")];
//! let client = nightfly::Client::new();
//! let res = client.post("http://httpbin.org/post")
//!     .form(&params)
//!     .send();
//! # Ok(())
//! # }
//! ```
//!
//! ### JSON
//!
//! There is also a `json` method helper on the [`RequestBuilder`][builder] that works in
//! a similar fashion the `form` method. It can take any value that can be
//! serialized into JSON. The feature `json` is required.
//!
//! ```rust
//! # use nightfly::Error;
//! # use std::collections::HashMap;
//! #
//! # fn run() -> Result<(), Error> {
//! // This will POST a body of `{"lang":"rust","body":"json"}`
//! let mut map = HashMap::new();
//! map.insert("lang", "rust");
//! map.insert("body", "json");
//!
//! let client = nightfly::Client::new();
//! let res = client.post("http://httpbin.org/post")
//!     .json(&map)
//!     .send();
//! # Ok(())
//! # }
//! ```
//!
//! ## Redirect Policies
//!
//! By default, a `Client` will automatically handle HTTP redirects, having a
//! maximum redirect chain of 10 hops. To customize this behavior, a
//! [`redirect::Policy`][redirect] can be used with a `ClientBuilder`.
//!
//! ## Cookies
//!
//! The automatic storing and sending of session cookies can be enabled with
//! the [`cookie_store`][ClientBuilder::cookie_store] method on `ClientBuilder`.
//!
//! ## Optional Features
//!
//! The following are a list of [Cargo features][cargo-features] that can be
//! enabled or disabled:
//!
//! - **cookies**: Provides cookie session support.
//!
//!
//! [client]: ./struct.Client.html
//! [response]: ./struct.Response.html
//! [get]: ./fn.get.html
//! [builder]: ./struct.RequestBuilder.html
//! [serde]: http://serde.rs
//! [redirect]: crate::redirect
//! [cargo-features]: https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section

pub use http::header;
pub use http::Method;
pub use http::StatusCode;
pub use http::{HeaderMap, HeaderValue};
pub use url::Url;

// universal mods
#[macro_use]
mod error;
mod into_url;
mod response;

pub use self::error::{Error, Result};
pub use self::into_url::IntoUrl;
pub use self::response::ResponseBuilderExt;

/// Shortcut method to quickly make a `GET` request.
///
/// See also the methods on the [`nightfly::Response`](./struct.Response.html)
/// type.
///
/// **NOTE**: This function creates a new internal `Client` on each call,
/// and so should not be used if making many requests. Create a
/// [`Client`](./struct.Client.html) instead.
///
/// # Examples
///
/// ```rust
/// # fn run() -> Result<(), nightfly::Error> {
/// let body = nightfly::get("https://www.rust-lang.org")
///     .text();
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This function fails if:
///
/// - native TLS backend cannot be initialized
/// - supplied `Url` cannot be parsed
/// - there was an error while sending request
/// - redirect limit was exhausted
pub fn get<T: IntoUrl>(url: T) -> crate::Result<HttpResponse> {
    Client::new().get(url).send()
}

fn _assert_impls() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_clone<T: Clone>() {}

    assert_send::<Client>();
    // assert_sync::<Client>();
    assert_clone::<Client>();

    assert_send::<Request>();
    assert_send::<RequestBuilder>();

    assert_send::<Error>();
    assert_sync::<Error>();
}

// #[cfg(test)]
// #[macro_use]
// extern crate doc_comment;

// #[cfg(test)]
// doctest!("../README.md");

// #[cfg(feature = "multipart")]
// pub use self::lunatic_impl::multipart;
pub use self::lunatic_impl::{
    Body, Client, ClientBuilder, HttpResponse, Request, RequestBuilder, SerializableResponse,
};
#[cfg(feature = "__tls")]
// Re-exports, to be removed in a future release
pub use tls::{Certificate, Identity};

#[cfg(feature = "cookies")]
pub mod cookie;
mod lunatic_impl;
pub mod redirect;
#[cfg(feature = "__tls")]
pub mod tls;
mod util;
mod version;
pub use version::Version;
