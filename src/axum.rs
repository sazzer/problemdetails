use std::any::Any;

use axum::{
    body::Body,
    http::header::CONTENT_TYPE,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tower_http::catch_panic::{CatchPanicLayer, ResponseForPanic};

use super::Problem;

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        if self.body.is_empty() {
            self.status_code.into_response()
        } else {
            let body = Json(self.body);
            let mut response = (self.status_code, body).into_response();

            response
                .headers_mut()
                .insert(CONTENT_TYPE, "application/problem+json".parse().unwrap());
            response
        }
    }
}

#[derive(Debug, Clone)]
pub struct PanicHandlerBuilder {
    fill_detail: bool,
    problem: Problem,
}

/// Create a builder for [`tower_http::catch_panic:CatchPanicLayer`] which transforms panics into RFC-7807-compatible responses.
impl PanicHandlerBuilder {
    pub fn new() -> Self {
        Self {
            fill_detail: cfg!(debug_assertions),
            problem: crate::new(StatusCode::INTERNAL_SERVER_ERROR)
                .with_title("Internal server error"),
        }
    }

    /// Enable automatic setting of the `Problem` `detail` field to the panic message.
    /// By default it is `true` for debug builds and `false` for release builds.
    pub fn with_fill_detail(mut self, enabled: bool) -> Self {
        self.fill_detail = enabled;
        return self;
    }

    /// Set the base problem to be used by the panic handler.
    /// If `fill_detail` is enabled, the `detail` field will be replaced with the panic message.
    pub fn with_problem(mut self, problem: Problem) -> Self {
        self.problem = problem;
        return self;
    }

    /// Build the PanicHandler.
    pub fn build(self) -> CatchPanicLayer<PanicHandlerBuilder> {
        CatchPanicLayer::custom(self)
    }
}

impl ResponseForPanic for PanicHandlerBuilder {
    type ResponseBody = Body;

    fn response_for_panic(
        &mut self,
        err: Box<dyn Any + Send + 'static>,
    ) -> http::Response<Self::ResponseBody> {
        let detail = if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Internal server error".to_string()
        };

        if self.fill_detail {
            self.problem.clone().with_detail(detail).into_response()
        } else {
            self.problem.clone().into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;
    use http::{header::CONTENT_TYPE, StatusCode};
    use insta::assert_json_snapshot;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn no_values() {
        let router = axum::Router::new().route(
            "/test",
            axum::routing::get(|| async { crate::new(StatusCode::BAD_REQUEST) }),
        );

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/test").await;

        check!(response.status_code() == StatusCode::BAD_REQUEST);
        check!(response.headers().get(CONTENT_TYPE) == None);
        check!(response.text() == "");
    }

    #[tokio::test]
    async fn default_panic() {
        let router = axum::Router::new()
            .route(
                "/panic",
                axum::routing::get(|| async { panic!("Panic message") }),
            )
            .layer(crate::axum::PanicHandlerBuilder::new().build());

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/panic").await;

        check!(response.status_code() == StatusCode::INTERNAL_SERVER_ERROR);
        check!(response.header(CONTENT_TYPE) == "application/problem+json");
        let body: Value = response.json();

        assert_json_snapshot!(body, @r###"
        {
          "detail": "Panic message",
          "title": "Internal server error"
        }
        "###);
    }

    #[tokio::test]
    async fn fill_panic_message_false() {
        let router = axum::Router::new()
            .route(
                "/panic",
                axum::routing::get(|| async { panic!("Panic message") }),
            )
            .layer(
                crate::axum::PanicHandlerBuilder::new()
                    .with_fill_detail(false)
                    .build(),
            );

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/panic").await;

        check!(response.status_code() == StatusCode::INTERNAL_SERVER_ERROR);
        check!(response.header(CONTENT_TYPE) == "application/problem+json");
        let body: Value = response.json();

        assert_json_snapshot!(body, @r###"
        {
          "title": "Internal server error"
        }
        "###);
    }

    #[tokio::test]
    async fn customized_panic() {
        let router = axum::Router::new()
            .route(
                "/panic",
                axum::routing::get(|| async { panic!("Panic message") }),
            )
            .layer(
                crate::axum::PanicHandlerBuilder::new()
                    .with_problem(
                        crate::new(StatusCode::IM_A_TEAPOT).with_instance("some instance"),
                    )
                    .build(),
            );

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/panic").await;

        check!(response.status_code() == StatusCode::IM_A_TEAPOT);
        check!(response.header(CONTENT_TYPE) == "application/problem+json");
        let body: Value = response.json();

        assert_json_snapshot!(body, @r###"
        {
          "detail": "Panic message",
          "instance": "some instance"
        }
        "###);
    }

    #[tokio::test]
    async fn rfc7807_forbidden_example() {
        let router: axum::Router = axum::Router::new().route(
            "/test",
            axum::routing::get(|| async {
                crate::new(StatusCode::FORBIDDEN)
                    .with_type("https://example.com/probs/out-of-credit")
                    .with_title("You do not have enough credit.")
                    .with_detail("Your current balance is 30, but that costs 50.")
                    .with_instance("/account/12345/msgs/abc")
                    .with_value("balance", 30)
                    .with_value("accounts", vec!["/account/12345", "/account/67890"])
            }),
        );

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/test").await;

        check!(response.status_code() == StatusCode::FORBIDDEN);
        check!(response.header(CONTENT_TYPE) == "application/problem+json");

        let body: Value = response.json();

        assert_json_snapshot!(body, @r###"
        {
          "accounts": [
            "/account/12345",
            "/account/67890"
          ],
          "balance": 30,
          "detail": "Your current balance is 30, but that costs 50.",
          "instance": "/account/12345/msgs/abc",
          "title": "You do not have enough credit.",
          "type": "https://example.com/probs/out-of-credit"
        }
        "###);
    }

    #[tokio::test]
    async fn rfc7807_validation_example() {
        let router: axum::Router = axum::Router::new().route(
            "/test",
            axum::routing::get(|| async {
                crate::new(StatusCode::FORBIDDEN)
                    .with_type("https://example.net/validation-error")
                    .with_title("Your request parameters didn't validate.")
                    .with_value(
                        "invalid-params",
                        json!([ {
              "name": "age",
              "reason": "must be a positive integer"
            },
            {
              "name": "color",
              "reason": "must be 'green', 'red' or 'blue'"}]),
                    )
            }),
        );

        let test_server = axum_test::TestServer::new(router).unwrap();

        let response = test_server.get("/test").await;

        check!(response.status_code() == StatusCode::FORBIDDEN);
        check!(response.header(CONTENT_TYPE) == "application/problem+json");

        let body: Value = response.json();

        assert_json_snapshot!(body, @r###"
        {
          "invalid-params": [
            {
              "name": "age",
              "reason": "must be a positive integer"
            },
            {
              "name": "color",
              "reason": "must be 'green', 'red' or 'blue'"
            }
          ],
          "title": "Your request parameters didn't validate.",
          "type": "https://example.net/validation-error"
        }
        "###);
    }
}
