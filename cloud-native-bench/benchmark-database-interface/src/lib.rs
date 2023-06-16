use std::error::Error;

use benchmark_adapter_types::ResultResponse;
use sqlx::{Connection, PgConnection, Row};

pub struct DatabaseInterface {}

impl DatabaseInterface {
    pub async fn save_results(
        database_url: String,
        result_response: ResultResponse,
    ) -> Result<(), Box<dyn Error>> {
        let mut conn = PgConnection::connect(&database_url).await?;

        let insert_result = sqlx::query(
            "INSERT INTO benchmark_results (name, description, raw_data) VALUES ($1, $2, $3) RETURNING id;",
        )
        .bind(result_response.name)
        .bind(result_response.description)
        .bind(result_response.raw_data)
        .fetch_one(&mut conn)
        .await?;

        let result_id: i32 = insert_result.try_get("id")?;

        for benchmark_data_entry in result_response.data {
            sqlx::query(
                "INSERT INTO benchmark_data(benchmark_results_id, parameter, data_unit, measurement_name, measurements) VALUES ($1, $2, $3, $4, $5);"
            )
            .bind(result_id)
            .bind(benchmark_data_entry.parameter)
            .bind(benchmark_data_entry.data_unit)
            .bind(benchmark_data_entry.measurement_name)
            .bind(benchmark_data_entry.measurements)
            .execute(&mut conn)
            .await?;
        }

        Ok(())
    }
}
