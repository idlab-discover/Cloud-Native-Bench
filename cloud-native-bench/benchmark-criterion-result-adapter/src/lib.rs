use benchmark_adapter_types::{DataEntry, ResultResponse};
use serde::Deserialize;

/// `cargo-criterion` JSON output struct needed to deserialize the result.
#[derive(Deserialize, Debug)]
struct CriterionJsonResponse {
    // pub reason: String,
    pub id: String,

    // report_directory: String,
    pub iteration_count: Vec<u64>,
    pub measured_values: Vec<f64>,
    pub unit: String,
    // throughput: Vec<Throughput>,

    // typical: ConfidenceInterval,
    // mean: ConfidenceInterval,
    // median: ConfidenceInterval,
    // median_abs_dev: ConfidenceInterval,
    // slope: Option<ConfidenceInterval>,

    // change: Option<ChangeDetails>,
}

/// Adapt the JSON output of `cargo criterion` to the Benchmark output that allows automated analysis.
/// `measurement_name` will be printed on the x-axis of the graphs (e.g., "time" or "memory").
pub fn adapt_critertion_results(
    raw_json_data: &str,
    benchmark_name: &str,
    benchmark_description: &str,
    measurement_name: &str,
) -> ResultResponse {
    let mut result_response = ResultResponse {
        name: benchmark_name.into(),
        description: benchmark_description.into(),
        data: Vec::new(),
        raw_data: raw_json_data.into(),
        generated_jupyter: None,
    };

    // Loop over every JSON object (one per line) in the raw results.
    for line in raw_json_data.lines() {
        // Deserialize `line` JSON object to struct.
        let json_response = serde_json::from_str::<CriterionJsonResponse>(line);

        // Only parse `benchmark-complete` JSON object, disregard `group-complete` JSON object.
        if let Ok(json_response) = json_response {
            // Divide the sample measurement with the amount of iterations were executed in the sample.
            let calculated_values: Vec<f64> = json_response
                .measured_values
                .iter()
                .zip(json_response.iteration_count.iter())
                .map(|(value, iterations)| *value / (*iterations as f64))
                .collect();

            // Create a data entry to be included in the results
            let data_entry = DataEntry {
                parameter: json_response.id,
                measurements: calculated_values,
                data_unit: json_response.unit.clone(),
                measurement_name: format!("{} ({})", measurement_name, json_response.unit),
            };

            result_response.data.push(data_entry);
        }
    }

    result_response
}
