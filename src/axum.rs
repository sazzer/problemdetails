use axum::{
    response::{IntoResponse, Response},
    Json,
};

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
                .insert("content-type", "application/problem+json".parse().unwrap());
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;
    use http::StatusCode;
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
        check!(response.headers().get("Content-Type") == None);
        check!(response.text() == "");
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
        check!(response.header("Content-Type") == "application/problem+json");

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
        check!(response.header("Content-Type") == "application/problem+json");

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
