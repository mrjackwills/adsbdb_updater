#![forbid(unsafe_code)]
#![warn(
    clippy::expect_used,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unused_async,
    clippy::unwrap_used
)]
#![allow(clippy::module_name_repetitions, clippy::doc_markdown)]
// Only allow when debugging
// #![allow(unused, clippy::todo)]

use std::io;

use app_error::AppError;
use parse_env::AppEnv;
use serde::{Deserialize, Serialize};

use crate::{
    callsign::{Callsign, Validate},
    db::{ModelAirport, ModelFlightroute},
};
mod app_error;
mod callsign;
mod db;
mod n_number;
mod parse_env;

fn setup_tracing(app_env: &AppEnv) {
    tracing_subscriber::fmt()
        .with_max_level(app_env.log_level)
        .init();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UpdatedFlightroute {
    callsign: String,
    origin: String,
    destination: String,
}

fn load_data_into_vec() -> Result<Vec<UpdatedFlightroute>, AppError> {
    let input = "./input.csv";
    let file_input = std::fs::File::open(input)?;
    let reader = io::BufReader::new(&file_input);
    let mut rdr = csv::Reader::from_reader(reader);
    Ok(rdr
        .deserialize::<UpdatedFlightroute>()
        .flatten()
        .collect::<Vec<UpdatedFlightroute>>())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let app_env = parse_env::AppEnv::get_env();
    setup_tracing(&app_env);
    let postgres = db::db_pool(&app_env).await?;
    let mut redis = db::get_connection(&app_env).await?;

    let flights_to_update = load_data_into_vec()?;

    for i in flights_to_update {
        let callsign = Callsign::validate(&i.callsign)?;
        if let Some(flightroute) = ModelFlightroute::get(&postgres, &callsign).await? {
            let original_airport = ModelAirport::get(&postgres, &i.origin).await?;
            let destination_airport = ModelAirport::get(&postgres, &i.destination).await?;
            if let (Some(origin), Some(destination)) = (original_airport, destination_airport) {
                flightroute
                    .update(&postgres, &mut redis, origin, destination)
                    .await?;
            }
        }
    }
    Ok(())
}
