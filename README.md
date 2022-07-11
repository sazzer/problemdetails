# Problem Details

[![Build status](https://github.com/sazzer/problemdetails/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sazzer/problemdetails/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/problemdetails)](https://crates.io/crates/problemdetails)
[![Documentation](https://docs.rs/problemdetails/badge.svg)](https://docs.rs/problemdetails)

This crate provides an implementation of a Problem Details response for HTTP APIs, as defined in [RFC-7807](https://datatracker.ietf.org/doc/html/rfc7807). This is a standard way for HTTP APIs to indicate that a problem occurred with the request, including some standard payload fields as required.

## Example Usage

The following is a valid handler for Axum that returns the Forbidden example from [RFC-7807 Section 3](https://datatracker.ietf.org/doc/html/rfc7807#section-3):
```rust
async fn forbidden() -> problemdetails::Result<String> {
    Err(problemdetails::new(StatusCode::FORBIDDEN)
            .with_type("https://example.com/probs/out-of-credit")
            .with_title("You do not have enough credit.")
            .with_detail("Your current balance is 30, but that costs 50.")
            .with_instance("/account/12345/msgs/abc")
            .with_value("balance", 30)
            .with_value("accounts", vec!["/account/12345", "/account/67890"]))
}
```

## Supported HTTP Servers

Currently this is only supported with the following HTTP Servers: 
- [Axum](https://crates.io/crates/axum)

Examples of use with the different HTTP Servers can be found in the [examples](https://github.com/sazzer/problemdetails/tree/main/examples) directory.

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## Minimum supported Rust version

The MSRV for `problemdetails` is 1.60.0. However, the HTTP Servers that are used with it might need a higher version.

## License

This project is licensed under the MIT license.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `problemdetails` by you, shall be licensed as MIT, without any additional terms or conditions.
