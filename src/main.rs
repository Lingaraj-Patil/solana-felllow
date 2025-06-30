use axum::{routing::post, Router};
use std::net::SocketAddr;

mod routes;
use routes::token::{create_token, mint_token};
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(routes::keypair::generate_keypair))
        .route("/token/create", post(routes::token::create_token))
        .route("/token/mint", post(routes::token::mint_token))
        .route("/message/sign", post(routes::message::sign_message))
        .route("/message/verify", post(routes::message::verify_message))
        .route("/send/sol", post(routes::transfer::send_sol))
        .route("/send/token", post(routes::transfer::send_token));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}