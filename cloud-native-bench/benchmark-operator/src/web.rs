use axum::{extract, routing::get, Json, Router};
use tokio::signal;
use tracing::info;

use crate::state::{State, StateData};

// TODO: this web server will probably be removed from the operator and moved to a separate microservice

pub async fn spawn(state: &State) {
    info!("Web server starting on on 0.0.0.0:3000");

    let app = Router::new()
        .route("/status", get(get_status))
        .with_state(state.clone());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// Routes

async fn get_status(extract::State(state): extract::State<State>) -> Json<StateData> {
    let state_data = state.state_data.read().await.clone();

    Json(state_data)
}

// Adapted from: https://github.com/tokio-rs/axum/blob/6377dbebc0db0e4e204c38dd84dc457c062146d1/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Axum received shutdown signal")
}
