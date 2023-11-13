use std::net::SocketAddr;

use sero::{config, prepare};

#[tokio::main]
async fn main() {
    let app = prepare().await;
    let config = config::Config::default();
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
