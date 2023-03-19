use crate::{app_error::AppError, parse_env::AppEnv};
use redis::{aio::Connection, ConnectionAddr, ConnectionInfo, RedisConnectionInfo};
use sqlx::{postgres::PgPoolOptions, ConnectOptions, PgPool};
use std::time::Duration;

mod model_airport;
mod model_flightroute;

pub use model_airport::ModelAirport;
pub use model_flightroute::ModelFlightroute;

pub async fn db_pool(app_env: &AppEnv) -> Result<PgPool, AppError> {
    let mut options = sqlx::postgres::PgConnectOptions::new()
        .host(&app_env.pg_host)
        .port(app_env.pg_port)
        .database(&app_env.pg_database)
        .username(&app_env.pg_user)
        .password(&app_env.pg_pass);

    let acquire_timeout = Duration::from_secs(5);
    let idle_timeout = Duration::from_secs(30);

    if app_env.log_level != tracing::Level::TRACE {
        options.disable_statement_logging();
    }

    Ok(PgPoolOptions::new()
        .max_connections(20)
        .idle_timeout(idle_timeout)
        .acquire_timeout(acquire_timeout)
        .connect_with(options)
        .await?)
}

/// Get an async redis connection
pub async fn get_connection(app_env: &AppEnv) -> Result<Connection, AppError> {
    let connection_info = ConnectionInfo {
        redis: RedisConnectionInfo {
            db: i64::from(app_env.redis_database),
            password: Some(app_env.redis_password.clone()),
            username: None,
        },
        addr: ConnectionAddr::Tcp(app_env.redis_host.clone(), app_env.redis_port),
    };
    let client = redis::Client::open(connection_info)?;
    match tokio::time::timeout(Duration::from_secs(10), client.get_async_connection()).await {
        Ok(con) => Ok(con?),
        Err(_) => Err(AppError::Internal("Unable to connect to redis".to_owned())),
    }
}
