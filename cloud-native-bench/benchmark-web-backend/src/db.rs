use crate::types::{BenchmarkListResponse, BenchmarkResult};
use sqlx::{Pool, Postgres};
use std::{error::Error, sync::Arc};

/// Get full list of completed Benchmarks
pub async fn get_benchmark_list(
    pool: Arc<Pool<Postgres>>,
) -> Result<Vec<BenchmarkListResponse>, Box<dyn Error>> {
    let benchmark_results = sqlx::query_as::<_, BenchmarkResult>("SELECT * FROM benchmark_results")
        .fetch_all(pool.as_ref())
        .await?;

    Ok(benchmark_results
        .iter()
        .map(|benchmark_result| BenchmarkListResponse {
            id: benchmark_result.id,
            timestamp: benchmark_result.timestamp,
            name: benchmark_result.name.clone(),
            description: benchmark_result.description.clone(),
        })
        .collect())
}

/// Get details of one Benchmark
pub async fn get_benchmark_detail(
    id: u32,
    pool: Arc<Pool<Postgres>>,
) -> Result<BenchmarkResult, Box<dyn Error>> {
    let benchmark =
        sqlx::query_as::<_, BenchmarkResult>("SELECT * FROM benchmark_results WHERE id = $1")
            .bind(id as i32)
            .fetch_one(pool.as_ref())
            .await?;

    Ok(benchmark)
}
