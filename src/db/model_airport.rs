use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::app_error::AppError;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelAirport {
    pub airport_id: i64,
}

impl ModelAirport {
    pub async fn get(db: &PgPool, airport_icao: &str) -> Result<Option<Self>, AppError> {
        let query = r#"
SELECT
    airport_id
FROM airport
LEFT JOIN airport_iata_code ai USING(airport_iata_code_id)
WHERE
    ai.iata_code = $1"#;
        Ok(sqlx::query_as::<_, Self>(query)
            .bind(airport_icao)
            .fetch_optional(db)
            .await?)
    }
}
