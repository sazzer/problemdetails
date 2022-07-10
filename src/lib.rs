#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::unused_async,
    clippy::unused_self
)]

mod axum;

use std::collections::BTreeMap;

use http::StatusCode;
use serde_json::Value;

/// Representation of a Problem error to return to the client.
pub struct Problem {
    status_code: StatusCode,
    body:        BTreeMap<String, Value>,
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
    ///
    /// # Parameters
    /// - `value` - The value to use for the "type"
    #[must_use]
    pub fn with_type<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("type", value.into())
    }

    /// Specify the "title" to use for the problem.
    ///
    /// # Parameters
    /// - `value` - The value to use for the "title"
    #[must_use]
    pub fn with_title<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("title", value.into())
    }

    /// Specify the "detail" to use for the problem.
    ///
    /// # Parameters
    /// - `value` - The value to use for the "detail"
    #[must_use]
    pub fn with_detail<S>(self, value: S) -> Self
    where
        S: Into<String>,
    {
        self.with_value("detail", value.into())
    }

    /// Specify the "instance" to use for the problem.
    ///
    /// # Parameters
    /// - `value` - The value to use for the "instance"
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

/// Result type where the error is always a `Problem`.
pub type Result<T> = std::result::Result<T, Problem>;

