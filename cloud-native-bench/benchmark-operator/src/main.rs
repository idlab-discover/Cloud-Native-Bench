use benchmark_operator::{benchmark_controller, grpc, state::State, web};
use dotenv::dotenv;

/// Runs the operator
#[tokio::main]
async fn main() {
    // .env import
    dotenv().ok();

    // Setup tracing subscriber
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let state = State::default();

    let web_server = web::spawn(&state);
    let grpc_server = grpc::spawn(&state);
    let controller = benchmark_controller::run(&state);

    tokio::join!(web_server, grpc_server, controller);
}
