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
    use axum::response::IntoResponse;
    use http::StatusCode;
    use insta::assert_json_snapshot;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn no_values() {
        let problem = crate::new(StatusCode::BAD_REQUEST);

        let response = problem.into_response();
        check!(response.status() == StatusCode::BAD_REQUEST);
        check!(response.headers().get("Content-Type") == None);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        check!(body.len() == 0);
    }

    #[tokio::test]
    async fn rfc7807_forbidden_example() {
        let problem = crate::new(StatusCode::FORBIDDEN)
            .with_type("https://example.com/probs/out-of-credit")
            .with_title("You do not have enough credit.")
            .with_detail("Your current balance is 30, but that costs 50.")
            .with_instance("/account/12345/msgs/abc")
            .with_value("balance", 30)
            .with_value("accounts", vec!["/account/12345", "/account/67890"]);

        let response = problem.into_response();
        check!(response.status() == StatusCode::FORBIDDEN);
        check!(response.headers().get("Content-Type").unwrap() == "application/problem+json");

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        check!(body.len() != 0);
        let body: Value = serde_json::from_slice(&body).unwrap();

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
        "###)
    }

    #[tokio::test]
    async fn rfc7807_validation_example() {
        let problem = crate::new(StatusCode::FORBIDDEN)
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
            );

        let response = problem.into_response();
        check!(response.status() == StatusCode::FORBIDDEN);
        check!(response.headers().get("Content-Type").unwrap() == "application/problem+json");

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        check!(body.len() != 0);
        let body: Value = serde_json::from_slice(&body).unwrap();

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
        "###)
    }
}

