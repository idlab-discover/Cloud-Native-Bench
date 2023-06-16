use serde::Serialize;

/// Standardized Benchmark data entry to be included in the result response.
#[derive(Debug, Serialize)]
pub struct DataEntry {
    pub parameter: String,
    pub measurements: Vec<f64>,
    pub data_unit: String,
    pub measurement_name: String,
}

/// Standardized Benchmark result response that allows for automated analysis on the data.
#[derive(Debug, Serialize)]
pub struct ResultResponse {
    pub name: String,
    pub description: String,
    pub data: Vec<DataEntry>,
    pub raw_data: String,
    pub generated_jupyter: Option<String>,
}
