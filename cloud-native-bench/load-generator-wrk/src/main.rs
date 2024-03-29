use std::{env, process::Command};

use benchmark_adapter_types::{DataEntry, ResultResponse};
use benchmark_database_interface::DatabaseInterface;
use benchmark_grpc::GrpcCall;
use dotenv::dotenv;
use hyper::{Client, Uri};
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Wait until the webserver is online
    let client = Client::new();
    let uri: Uri = env::var("WRK_ENDPOINT").unwrap().parse()?;

    let mut is_up = false;

    println!(r#"Checking if endpoint "{}" is up."#, uri.clone());

    while !is_up {
        let resp = client.get(uri.clone()).await;

        if resp.is_ok() {
            println!("Endpoint is up.");
            is_up = true;
        } else {
            println!("Timeout, retrying...");
        }
    }

    // Connect to the operator gRPC server and call `benchmark_started`.
    let mut grpc_call = GrpcCall::connect(
        env::var("OPERATOR_GRPC_ADDRESS")
            .expect("OPERATOR_GRPC_ADDRESS environment variable was not set."),
    )
    .await?;
    let database_url = grpc_call.benchmark_started().await?;

    // Run the benchmark and capture the results.
    let result_response = wrk_load_test();

    // Store the results in the database.
    DatabaseInterface::save_results(database_url, result_response).await?;

    // Call `benchmark_done` to mark this benchmark as done.
    let ack = grpc_call.benchmark_done().await?;
    println!("Benchmark done response: {}", ack);

    Ok(())
}

fn wrk_load_test() -> ResultResponse {
    let mut req_sec_de = DataEntry {
        parameter: format!(
            "{} time(s): 'wrk -c {} -t {} -d {} --timeout {} {}'",
            env::var("ITERATIONS").unwrap_or("5".to_string()),
            env::var("WRK_CONNECTIONS").unwrap_or("400".to_string()),
            env::var("WRK_THREADS").unwrap_or("8".to_string()),
            env::var("WRK_DURATION").unwrap_or("1m".to_string()),
            env::var("WRK_TIMEOUT").unwrap_or("10s".to_string()),
            env::var("WRK_ENDPOINT").unwrap()
        ),
        measurement_name: "Requests per second (req/s)".to_string(),
        data_unit: "req/s".to_string(),
        measurements: Vec::new(),
    };

    let mut transfer_sec_de = DataEntry {
        parameter: format!(
            "{} time(s): 'wrk -c {} -t {} -d {} --timeout {} {}'",
            env::var("ITERATIONS").unwrap_or("5".to_string()),
            env::var("WRK_CONNECTIONS").unwrap_or("400".to_string()),
            env::var("WRK_THREADS").unwrap_or("8".to_string()),
            env::var("WRK_DURATION").unwrap_or("1m".to_string()),
            env::var("WRK_TIMEOUT").unwrap_or("10s".to_string()),
            env::var("WRK_ENDPOINT").unwrap()
        ),
        measurement_name: String::new(),
        data_unit: String::new(),
        measurements: Vec::new(),
    };

    let mut raw_data = String::new();

    let req_sec_re = Regex::new(r"Requests/sec:\s*(\d+\.?\d*)").unwrap();
    let transfer_sec_re = Regex::new(r"Transfer/sec:\s*(\d+\.?\d*)(\w*)").unwrap();

    for _ in 1..=env::var("ITERATIONS")
        .unwrap_or("5".to_string())
        .parse()
        .unwrap()
    {
        let result = Command::new("wrk")
            .args([
                "-c",
                &env::var("WRK_CONNECTIONS").unwrap_or("400".to_string()),
                "-t",
                &env::var("WRK_THREADS").unwrap_or("8".to_string()),
                "-d",
                &env::var("WRK_DURATION").unwrap_or("1m".to_string()),
                "--timeout",
                &env::var("WRK_TIMEOUT").unwrap_or("10s".to_string()),
                &env::var("WRK_ENDPOINT").unwrap(),
            ])
            .output()
            .unwrap();

        let result = std::str::from_utf8(&result.stdout).unwrap().to_string();

        let captures = req_sec_re.captures(&result);

        if let Some(captures) = captures {
            req_sec_de
                .measurements
                .push(captures.get(1).unwrap().as_str().parse::<f64>().unwrap());
        }

        let captures = transfer_sec_re.captures(&result);

        if let Some(captures) = captures {
            if transfer_sec_de.measurement_name.is_empty() {
                // First measurement determines the data_unit and measurement_name
                transfer_sec_de.data_unit = format!("{}/s", captures.get(2).unwrap().as_str());
                transfer_sec_de.measurement_name =
                    format!("Transfer per second ({}/s)", transfer_sec_de.data_unit);
            }

            transfer_sec_de
                .measurements
                .push(captures.get(1).unwrap().as_str().parse::<f64>().unwrap());
        }

        raw_data += &result;
    }

    ResultResponse {
        name: env::var("TEST_NAME").unwrap_or("wrk load test.".to_string()),
        description: env::var("TEST_DESC").unwrap_or("wrk load test.".to_string()),
        data: vec![req_sec_de, transfer_sec_de],
        raw_data,
        generated_jupyter: None,
    }
}
