use benchmark_adapter_types::{DataEntry, ResultResponse};
use benchmark_database_interface::DatabaseInterface;
use benchmark_grpc::GrpcCall;
use dotenv::dotenv;
use regex::Regex;
use std::{env, process::Command, vec};

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
    let result_response = benchmark_linux_kernel();

    // Store the results in the database.
    DatabaseInterface::save_results(database_url, result_response).await?;

    // Call `benchmark_done` to mark this benchmark as done.
    let ack = grpc_call.benchmark_done().await?;
    println!("Benchmark done response: {}", ack);

    Ok(())
}

fn benchmark_linux_kernel() -> ResultResponse {
    // Start `kcbench` to compile Linux Kernel 6.2, 10 times, with 4 jobs

    let result = Command::new("kcbench")
        .args(["-s", "6.2", "-i", "10", "-j", "4"])
        .output()
        .unwrap();

    let result = std::str::from_utf8(&result.stdout).unwrap().to_string();

    // Covert result into expected format.
    let mut data_entry = DataEntry {
        parameter: "kcbench -s 6.2 -i 10 -j 4".into(),
        measurement_name: String::new(),
        measurements: Vec::new(),
        data_unit: String::new(),
    };

    let re = Regex::new(r"Run\s+(\d+)\s+\(-j\s+(\d+)\):\s+(\d+\.?\d*)\s+(\w+).*").unwrap();

    for result_line in result.lines() {
        let captures = re.captures(result_line);

        if let Some(captures) = captures {
            // Safe unwrap because the conditions for returning None will not occur.
            if captures.get(1).unwrap().as_str() == "1" {
                // First job, this job will determine the `data_unit` and we assume that the unit will be the same for the subsequential runs.
                data_entry.data_unit = captures.get(4).unwrap().as_str().into();
                data_entry.measurement_name =
                    format!("Compilation time ({})", data_entry.data_unit);
            }

            data_entry
                .measurements
                .push(captures.get(3).unwrap().as_str().parse::<f64>().unwrap());
        }
    }

    ResultResponse {
        name: "kcbench CPU Benchmark.".into(),
        description: "This benchmark will compile the Linux kernel a couple of times, testing CPU performance.".into(),
        data: vec![data_entry],
        raw_data: result,
        generated_jupyter: None
    }
}
