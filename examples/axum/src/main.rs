use std::net::SocketAddr;

use axum::{routing::get, Router};
use http::StatusCode;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(notimplemented))
        .route("/forbidden", get(forbidden));

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
