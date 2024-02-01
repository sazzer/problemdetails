#![deny(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]
#![allow(
    clippy::module_name_repetitions,
    clippy::unused_async,
    clippy::unused_self,
    clippy::ignored_unit_patterns
)]

//! This crate provides an implementation of a Problem Details response for HTTP APIs, as defined
//! in [RFC-7807](https://datatracker.ietf.org/doc/html/rfc7807) /
//! [RFC-9457](https://datatracker.ietf.org/doc/html/rfc9457). This is a standard way for HTTP
//! APIs to indicate that a problem occurred with the request, including some standard payload
//! fields as required.
//!
//! When used with a supported HTTP Server, this will automatically generate the correct JSON
//! response and set the Content-Type header to the correct value of `application/problem+json`.
//!
//! If used with an unsupported HTTP Server, the status code and body of the problem details can be
//! extracted and sent manually. The `body` field is in the correct structure to format into JSON
//! using something like `serde` already, so serializing it should be as simple as the HTTP Server
//! allows for.
//!
//! # Examples
//! ## Create an empty problem.
//! ```
//! # use http::StatusCode;
//! problemdetails::new(StatusCode::BAD_REQUEST);
//! ```
//! ## Create a populated problem.
//! ```
//! # use http::StatusCode;
//! problemdetails::new(StatusCode::FORBIDDEN)
//!     .with_type("https://example.com/probs/out-of-credit")
//!     .with_title("You do not have enough credit.")
//!     .with_detail("Your current balance is 30, but that costs 50.")
//!     .with_instance("/account/12345/msgs/abc")
//!     .with_value("balance", 30)
//!     .with_value("accounts", vec!["/account/12345", "/account/67890"]);
//! ```
//! # Features
//! HTTP Server support is behind feature flags for the appropriate HTTP Server. As such, you will
//! need to enable the correct feature for the HTTP Server that you are using.
//!
//! Currently supported features are:
//! * `axum` - For the [Axum](https://crates.io/crates/axum) HTTP Server.

#[cfg(feature = "axum")]
pub mod axum;

use std::collections::BTreeMap;

use http::StatusCode;
use serde_json::Value;

/// Representation of a Problem error to return to the client.
#[allow(dead_code)] // These fields are used by the various features.
#[derive(Debug, Clone)]
pub struct Problem {
    /// The status code of the problem.
    pub status_code: StatusCode,
    /// The actual body of the problem.
    pub body:        BTreeMap<String, Value>,
}

/// Create a new `Problem` response to send to the client.

#[must_use]
pub fn new<S>(status_code: S) -> Problem
where
    S: Into<StatusCode>,
{
    Problem {
        status_code: status_code.into(),
        body:        BTreeMap::new(),
    }
}

impl Problem {
    /// Specify the "type" to use for the problem.
    #[must_use]
    pub fn with_type<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("type", value.into())
    }

    /// Specify the "title" to use for the problem.
    #[must_use]
    pub fn with_title<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("title", value.into())
    }

    /// Specify the "detail" to use for the problem.
    #[must_use]
    pub fn with_detail<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("detail", value.into())
    }

    /// Specify the "instance" to use for the problem.
    #[must_use]
    pub fn with_instance<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("instance", value.into())
    }

    /// Specify an arbitrary value to include in the problem.
    ///
    /// # Parameters
    /// - `key` - The key for the value.
    /// - `value` - The value itself.
    #[must_use]
    pub fn with_value<V>(mut self, key: &str, value: V) -> Self
    where
        V: Into<Value>,
    {
        self.body.insert(key.to_owned(), value.into());

        self
    }
}

impl<S> From<S> for Problem
where
    S: Into<StatusCode>,
{
    fn from(status_code: S) -> Self {
        new(status_code.into())
    }
}
/// Result type where the error is always a `Problem`.
pub type Result<T> = std::result::Result<T, Problem>;
