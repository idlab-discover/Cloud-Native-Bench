use crate::{
    db,
    types::{
        BenchmarkDetailsTemplate, BenchmarkListResponse, BenchmarkListTemplate, IndexResponse,
    },
};
use askama::Template;
use axum::{extract::Path, http::header, Json};
use axum::{extract::State, response::IntoResponse};
use axum::{http::StatusCode, response::Html};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// Returns some basic information about this service
pub async fn index() -> Json<IndexResponse> {
    Json(IndexResponse {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        contact: env!("CARGO_PKG_AUTHORS").into(),
    })
}

/// Returns a list of all the benchmark results
pub async fn api_list_benchmark_results(
    State(pool): State<Arc<Pool<Postgres>>>,
) -> Result<Json<Vec<BenchmarkListResponse>>, StatusCode> {
    let benchmark_results = db::get_benchmark_list(pool.clone()).await;

    if let Ok(benchmark_results) = benchmark_results {
        Ok(Json(benchmark_results))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

/// Render an HTML page with a list of all the completed Benchmarks
pub async fn render_list_benchmark_results(
    State(pool): State<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let benchmark_results = db::get_benchmark_list(pool.clone()).await;

    if let Ok(benchmark_results) = benchmark_results {
        let template = BenchmarkListTemplate {
            benchmarks: benchmark_results,
        };

        match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render Benchmark list template: {}.", err),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve data from DB.",
        )
            .into_response()
    }
}

/// Render an HTML page with details of a Benchmark
pub async fn render_benchmark_details(
    Path(id): Path<u32>,
    State(pool): State<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let benchmark_details = db::get_benchmark_detail(id, pool).await;

    if let Ok(benchmark_details) = benchmark_details {
        let template = BenchmarkDetailsTemplate {
            benchmark: benchmark_details,
        };

        match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render Benchmark details: {}.", err),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve data from DB.",
        )
            .into_response()
    }
}

/// Download the raw data as a txt file
pub async fn download_raw_data(
    Path(id): Path<u32>,
    State(pool): State<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let benchmark_details = db::get_benchmark_detail(id, pool).await;

    if let Ok(benchmark_details) = benchmark_details {
        let headers = [
            (header::CONTENT_TYPE, "text/txt; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"benchmark_{}_raw_data.txt\"", id),
            ),
        ];

        (headers, benchmark_details.raw_data).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve data from DB.",
        )
            .into_response()
    }
}

/// Download the jupyter notebook
pub async fn download_jupyter(
    Path(id): Path<u32>,
    State(pool): State<Arc<Pool<Postgres>>>,
) -> impl IntoResponse {
    let benchmark_details = db::get_benchmark_detail(id, pool).await;

    if let Ok(benchmark_details) = benchmark_details {
        if let Some(generated_jupyter) = benchmark_details.generated_jupyter {
            let headers = [
                (header::CONTENT_TYPE, "text/json; charset=utf-8".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"benchmark_{}_jupyter.ipynb\"", id),
                ),
            ];

            (headers, generated_jupyter).into_response()
        } else {
            (StatusCode::NOT_FOUND).into_response()
        }
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve data from DB.",
        )
            .into_response()
    }
}
