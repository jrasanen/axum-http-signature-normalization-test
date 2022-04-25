use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use http_signature_normalization_reqwest::prelude::*;
use reqwest::{header::DATE, Client, StatusCode};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[debug_handler]
async fn root() -> Result<Html<String>, MyError> {
    let config = Config::default().require_header("accept");

    let response = Client::new()
        .post("https://example.org/")
        .header("User-Agent", "Reqwest")
        .header("Accept", "text/plain")
        .signature_with_digest(config, "my-key-id", Sha256::new(), "my request body", |s| {
            println!("Signing String\n{}", s);
            Ok(base64::encode(s)) as Result<_, MyError>
        })
        .await
        .unwrap();

    println!("{:?}", response);

    Ok(Html("Hello, World!".to_string()))
}

impl IntoResponse for MyError {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Failed to create signing string, {0}")]
    Convert(#[from] SignError),

    #[error("Failed to send request")]
    SendRequest(#[from] reqwest::Error),

    #[error("Failed to retrieve request body")]
    Body(reqwest::Error),
}
