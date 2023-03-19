use redis::RedisError;
use std::{io, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid callsign:")]
    Callsign(String),
    #[error("internal error:")]
    Internal(String),
    #[error("invalid modeS:")]
    ModeS(String),
    #[error("invalid n_number:")]
    NNumber(String),
    #[error("redis error")]
    RedisError(#[from] RedisError),
    #[error("not found")]
    SqlxError(#[from] sqlx::Error),
    #[error("parse int")]
    ParseInt(#[from] ParseIntError),
    #[error("IO error")]
    IoErrir(#[from] io::Error),
}
