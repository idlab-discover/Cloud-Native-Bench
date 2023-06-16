use crate::routes::{
    api_list_benchmark_results, download_jupyter, download_raw_data, index,
    render_benchmark_details, render_list_benchmark_results,
};
use axum::routing::get;
use axum::{Router, Server};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tokio::signal;

pub mod db;
pub mod routes;
pub mod types;

#[tokio::main]
async fn main() {
    dotenv().ok();

    println!("Server starting on {}.", env::var("ENDPOINT").unwrap());

    // Create a connection pool to connect to Postgres.
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    // Wrap pool inside an atomically reference counted pointer (thread safe).
    let shared_pool = Arc::new(pool);

    // Create the router and attach the routes.
    let app = Router::new()
        .route("/", get(index))
        // API
        .route("/api/benchmark-results", get(api_list_benchmark_results))
        .route(
            "/api/benchmark-results/:id/download/raw",
            get(download_raw_data),
        )
        .route(
            "/api/benchmark-results/:id/download/jupyter",
            get(download_jupyter),
        )
        // HTML render
        .route("/benchmark-results", get(render_list_benchmark_results))
        .route("/benchmark-results/:id", get(render_benchmark_details))
        .with_state(shared_pool);

    // Start the server.
    Server::bind(&env::var("ENDPOINT").unwrap().parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            // Adapted from: https://github.com/tokio-rs/axum/blob/6377dbebc0db0e4e204c38dd84dc457c062146d1/examples/graceful-shutdown/src/main.rs
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
        })
        .await
        .unwrap();
}
