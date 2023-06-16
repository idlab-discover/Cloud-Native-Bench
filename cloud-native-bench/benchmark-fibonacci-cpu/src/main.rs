use benchmark_database_interface::DatabaseInterface;
use benchmark_grpc::GrpcCall;
use dotenv::dotenv;
use std::{env, process::Command};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Connect to the operator gRPC server and call `benchmark_started`.
    let mut grpc_call = GrpcCall::connect(
        env::var("OPERATOR_GRPC_ADDRESS")
            .expect("OPERATOR_GRPC_ADDRESS environment variable was not set."),
    )
    .await?;
    let database_url = grpc_call.benchmark_started().await?;

    // Run the benchmark and capture the results.
    let results = benchmark_fibonacci();

    let result_response = benchmark_criterion_result_adapter::adapt_critertion_results(results.as_str(), "Fibonacci benchmark", "This benchmark will run the fibonacci calculation for [5, 6, 7, 8, 9, 10], 100 samples each, each sample consisting of a lot (worst case only one) of iterations.", "Time");

    // Store the results in the database.
    DatabaseInterface::save_results(database_url, result_response).await?;

    // Call `benchmark_done` to mark this benchmark as done.
    let ack = grpc_call.benchmark_done().await?;
    println!("Benchmark done response: {}", ack);

    Ok(())
}

fn benchmark_fibonacci() -> String {
    let result = Command::new("cargo")
        .args(["criterion", "--message-format=json"])
        .output()
        .unwrap();

    std::str::from_utf8(&result.stdout).unwrap().to_string()
}
