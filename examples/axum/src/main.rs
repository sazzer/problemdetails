use std::net::SocketAddr;

use axum::{
    extract::{
        rejection::{self, BytesRejection, JsonRejection},
        FromRequest,
    },
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use http::StatusCode;
use problemdetails::{axum::PanicHandlerBuilder, Problem};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(notimplemented))
        .route("/forbidden", get(forbidden))
        .route("/panic", get(panic))
        .route("/json", post(json));

    // add a panic handler if you want one
    let app = app.layer(
        PanicHandlerBuilder::new()
            .with_problem(problemdetails::new(StatusCode::IM_A_TEAPOT))
            .build(),
    );

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn notimplemented() -> problemdetails::Problem {
    problemdetails::new(StatusCode::NOT_IMPLEMENTED)
}

async fn forbidden() -> Result<String, problemdetails::Problem> {
    Err(problemdetails::new(StatusCode::FORBIDDEN)
        .with_type("https://example.com/probs/out-of-credit")
        .with_title("You do not have enough credit.")
        .with_detail("Your current balance is 30, but that costs 50.")
        .with_instance("/account/12345/msgs/abc")
        .with_value("balance", 30)
        .with_value("accounts", vec!["/account/12345", "/account/67890"]))
}

async fn panic() {
    panic!("Oh no!");
}

// example how to handle JSON rejection and return a problemdetail

// some example JSON deserializable data
#[derive(Debug, Deserialize)]
struct ExampleData {
    name: String,
}

// an example handler with our special MyJson extractor
async fn json(MyJson(data): MyJson<ExampleData>) {
    println!("{:?}", data);
    println!("{}", data.name);
}

pub struct MyRejection(Problem);

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(MyRejection))]
pub struct MyJson<T>(pub T);

impl IntoResponse for MyRejection {
    fn into_response(self) -> axum::response::Response {
        return self.0.into_response();
    }
}

impl From<JsonRejection> for MyRejection {
    fn from(rejection: rejection::JsonRejection) -> Self {
        let (status, body, title) = match rejection {
            JsonRejection::JsonDataError(err) => {
                (err.status(), err.body_text(), "JSON deserialization error")
            },
            JsonRejection::JsonSyntaxError(err) => {
                (err.status(), err.body_text(), "JSON syntax error")
            },
            JsonRejection::MissingJsonContentType(err) => {
                (err.status(), err.body_text(), "Content type error")
            },
            JsonRejection::BytesRejection(BytesRejection::FailedToBufferBody(err)) => {
                (err.status(), err.body_text(), "Request buffering error")
            },
            _ => (StatusCode::BAD_GATEWAY, "".to_string(), "Unknown error"),
        };

        MyRejection(
            problemdetails::new(status)
                .with_title(body)
                .with_detail(title),
        )
    }
}
