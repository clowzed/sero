use sero::{app, enable_logging};
use std::net::SocketAddr;
use tokio::{net::TcpListener, signal};

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let _guard = enable_logging().await.expect("Failed to initialize logging");

    let (app, state) = match app().await {
        Ok((app, state)) => (app, state),
        Err(reason) => {
            tracing::error!(%reason, "Error occurred while initializing app!");
            return;
        }
    };

    let ip = [0, 0, 0, 0];
    let port = state.configuration().port();

    let addr = SocketAddr::from((ip, port));

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(cause) => {
            tracing::error!(%cause, "Failed to initialize tcp listener!");
            return;
        }
    };
    let service = app.into_make_service();

    tracing::info!("Server is now up and listening on {}", addr);

    axum::serve(listener, service)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .inspect_err(|cause| tracing::error!(%cause, "Failed to start or execute server!"))
        .ok();
}
