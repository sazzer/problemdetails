use http::{header::CONTENT_TYPE, HeaderMap};

use super::Problem;

impl poem::IntoResponse for Problem {
    fn into_response(self) -> poem::Response {
        if self.body.is_empty() {
            self.status_code.into_response()
        } else {
            let body = poem::web::Json(self.body.clone());

            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                "application/problem+json"
                    .parse()
                    .expect("Could not parse the content type for problem detail"),
            );

            (self.status_code, headers, body).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::check;
    use http::{header::CONTENT_TYPE, StatusCode};
    use insta::assert_json_snapshot;
    use poem::{get, handler, test::TestClient, Route};

    #[handler]
    fn no_value_handler() -> crate::Problem {
        crate::new(StatusCode::BAD_REQUEST)
    }

    #[handler]
    fn rfc7807_forbidden_example_handler() -> crate::Problem {
        crate::new(StatusCode::FORBIDDEN)
            .with_type("https://example.com/probs/out-of-credit")
            .with_title("You do not have enough credit.")
            .with_detail("Your current balance is 30, but that costs 50.")
            .with_instance("/account/12345/msgs/abc")
            .with_value("balance", 30)
            .with_value("accounts", vec!["/account/12345", "/account/67890"])
    }

    #[handler]
    fn rfc7807_validation_example_handler() -> crate::Problem {
        crate::new(StatusCode::FORBIDDEN)
            .with_type("https://example.net/validation-error")
            .with_title("Your request parameters didn't validate.")
            .with_value(
                "invalid-params",
                serde_json::json!([
                    {
                        "name": "age",
                        "reason": "must be a positive integer"
                    },
                    {
                        "name": "color",
                        "reason": "must be 'green', 'red' or 'blue'"
                    }
                ]),
            )
    }

    #[tokio::test]
    async fn no_values() {
        let app = Route::new().at("/test", get(no_value_handler));
        let cli = TestClient::new(app);

        let mut response = cli.get("/test").send().await;

        check!(response.0.status() == StatusCode::BAD_REQUEST);
        check!(response.0.headers().get(CONTENT_TYPE) == None);
        check!(response.0.take_body().is_empty());
    }

    #[tokio::test]
    async fn rfc7807_forbidden_example() {
        let app = Route::new().at("/test", get(rfc7807_forbidden_example_handler));
        let cli = TestClient::new(app);

        let mut response = cli.get("/test").send().await;

        check!(response.0.status() == StatusCode::FORBIDDEN);
        check!(response.0.content_type() == Some("application/problem+json"));

        let body = response
            .0
            .take_body()
            .into_json::<serde_json::Value>()
            .await
            .unwrap();

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
        let app = Route::new().at("/test", get(rfc7807_validation_example_handler));
        let cli = TestClient::new(app);

        let mut response = cli.get("/test").send().await;

        check!(response.0.status() == StatusCode::FORBIDDEN);
        check!(response.0.content_type() == Some("application/problem+json"));

        let body = response
            .0
            .take_body()
            .into_json::<serde_json::Value>()
            .await
            .unwrap();

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
