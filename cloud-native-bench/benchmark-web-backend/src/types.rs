use askama::Template;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

/// SQLx BenchmarkResult.
#[derive(FromRow, Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub raw_data: String,
    pub timestamp: DateTime<Utc>,
    pub generated_jupyter: Option<String>,
}

// Askama rendering templates.

/// Template type for displaying a list of completed Benchmarks.
#[derive(Template)]
#[template(path = "benchmark_list.html")]
pub struct BenchmarkListTemplate {
    pub benchmarks: Vec<BenchmarkListResponse>,
}

/// Template type for displaying details of a Benchmark.
#[derive(Template)]
#[template(path = "benchmark_details.html")]
pub struct BenchmarkDetailsTemplate {
    pub benchmark: BenchmarkResult,
}

// HTTP response types.

/// Response for index page.
#[derive(Serialize)]
pub struct IndexResponse {
    pub name: String,
    pub version: String,
    pub contact: String,
}

/// Data of each item that will be in a Benchmark list request.
#[derive(Serialize)]
pub struct BenchmarkListResponse {
    pub id: i32,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub description: String,
}
