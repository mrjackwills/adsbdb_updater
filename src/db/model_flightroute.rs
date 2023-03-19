use redis::{aio::Connection, AsyncCommands};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};

use crate::{app_error::AppError, callsign::Callsign};

use super::ModelAirport;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelFlightroute {
    pub flightroute_id: i64,
    pub callsign: String,
    pub callsign_iata: Option<String>,
    pub callsign_icao: Option<String>,

    pub airline_name: Option<String>,
    pub airline_country_name: Option<String>,
    pub airline_country_iso_name: Option<String>,
    pub airline_callsign: Option<String>,
    pub airline_icao: Option<String>,
    pub airline_iata: Option<String>,

    pub origin_airport_country_iso_name: String,
    pub origin_airport_country_name: String,
    pub origin_airport_elevation: i32,
    // THIS CAN BE NULL!
    pub origin_airport_iata_code: String,
    pub origin_airport_icao_code: String,
    pub origin_airport_latitude: f64,
    pub origin_airport_longitude: f64,
    pub origin_airport_municipality: String,
    pub origin_airport_name: String,

    pub midpoint_airport_country_iso_name: Option<String>,
    pub midpoint_airport_country_name: Option<String>,
    pub midpoint_airport_elevation: Option<i32>,
    pub midpoint_airport_iata_code: Option<String>,
    pub midpoint_airport_icao_code: Option<String>,
    pub midpoint_airport_latitude: Option<f64>,
    pub midpoint_airport_longitude: Option<f64>,
    pub midpoint_airport_municipality: Option<String>,
    pub midpoint_airport_name: Option<String>,

    pub destination_airport_country_iso_name: String,
    pub destination_airport_country_name: String,
    pub destination_airport_elevation: i32,
    pub destination_airport_iata_code: String,
    pub destination_airport_icao_code: String,
    pub destination_airport_latitude: f64,
    pub destination_airport_longitude: f64,
    pub destination_airport_municipality: String,
    pub destination_airport_name: String,
}

impl ModelFlightroute {
    /// Query for a fully joined Option<ModelFlightRoute>
    /// Don't return result, as issues with nulls in the database, that I can't be bothered to deal with at the moment
    async fn _get(db: &mut Transaction<'_, Postgres>, callsign: &Callsign) -> Option<Self> {
        let query = match callsign {
            Callsign::Iata(_) => Self::get_query_iata(),
            Callsign::Icao(_) => Self::get_query_icao(),
            Callsign::Other(_) => Self::get_query_callsign(),
        };

        match callsign {
            Callsign::Other(callsign) => sqlx::query_as::<_, Self>(query)
                .bind(callsign)
                .fetch_optional(&mut *db)
                .await
                .unwrap_or(None),
            Callsign::Iata(x) | Callsign::Icao(x) => {
                if let Ok(flightroute) = sqlx::query_as::<_, Self>(query)
                    .bind(&x.0)
                    .bind(&x.1)
                    .fetch_optional(&mut *db)
                    .await
                {
                    if let Some(flightroute) = flightroute {
                        Some(flightroute)
                    } else {
                        sqlx::query_as::<_, Self>(Self::get_query_callsign())
                            .bind(format!("{}{}", x.0, x.1))
                            .fetch_optional(&mut *db)
                            .await
                            .unwrap_or(None)
                    }
                } else {
                    None
                }
            }
        }
    }

    // Why is this a transaction?
    pub async fn get(db: &PgPool, callsign: &Callsign) -> Result<Option<Self>, AppError> {
        let mut transaction = db.begin().await?;
        let output = Self::_get(&mut transaction, callsign).await;
        transaction.commit().await?;
        Ok(output)
    }

    /// Query a flightroute based on a callsign with is a valid N-Number
    const fn get_query_callsign() -> &'static str {
        r"
SELECT
    fl.flightroute_id,
    $1 AS callsign,
    NULL AS callsign_iata,
    NULL AS callsign_icao,

    NULL as airline_name,
    NULL as airline_callsign,
    NULL as airline_iata,
    NULL as airline_icao,
    NULL as airline_country_name,
    NULL as airline_country_iso_name,

    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_longitude,
        
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_longitude,
            
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_longitude
FROM
    flightroute fl
LEFT JOIN flightroute_callsign flc USING(flightroute_callsign_id)
LEFT JOIN 
    flightroute_callsign_inner fci
ON
    fci.flightroute_callsign_inner_id = flc.callsign_id
LEFT JOIN airport apo ON apo.airport_id = fl.airport_origin_id
LEFT JOIN airport apm ON apm.airport_id = fl.airport_midpoint_id
LEFT JOIN airport apd ON apd.airport_id = fl.airport_destination_id

WHERE fci.callsign = $1 LIMIT 1"
    }

    /// Query a flightroute based on a callsign with is a valid ICAO callsign
    const fn get_query_icao() -> &'static str {
        r"
SELECT
    fl.flightroute_id,
    concat($1,$2) as callsign,
    concat(ai.iata_prefix, (SELECT callsign FROM flightroute_callsign_inner WHERE flightroute_callsign_inner_id = iata_prefix_id)) AS callsign_iata,
    concat(ai.icao_prefix, (SELECT callsign FROM flightroute_callsign_inner WHERE flightroute_callsign_inner_id = icao_prefix_id)) AS callsign_icao,
    
    (SELECT country_iso_name FROM COUNTRY where country_id = ai.country_id) as airline_country_iso_name,
    (SELECT country_name FROM COUNTRY where country_id = ai.country_id) as airline_country_name,
    ai.airline_callsign,
    ai.airline_name,
    ai.iata_prefix AS airline_iata,
    ai.icao_prefix AS airline_icao,
    
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_longitude,
          
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_longitude,
            
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_longitude
FROM
    flightroute fl
LEFT JOIN flightroute_callsign flc USING(flightroute_callsign_id)
LEFT JOIN 
    flightroute_callsign_inner fci
ON
    fci.flightroute_callsign_inner_id = flc.callsign_id
LEFT JOIN airline ai USING(airline_id)
LEFT JOIN airport apo ON apo.airport_id = fl.airport_origin_id
LEFT JOIN airport apm ON apm.airport_id = fl.airport_midpoint_id
LEFT JOIN airport apd ON apd.airport_id = fl.airport_destination_id
WHERE 
    flc.airline_id = (SELECT airline_id FROM airline WHERE icao_prefix = $1)
AND
    flc.icao_prefix_id = (SELECT flightroute_callsign_inner_id FROM flightroute_callsign_inner WHERE callsign = $2 LIMIT 1)"
    }

    /// Query a flightroute based on a callsign with is a valid IATA callsign
    /// The `DISTINCT` subquery is bad, and will crash!
    /// Limit 1?
    const fn get_query_iata() -> &'static str {
        r"
SELECT
    fl.flightroute_id,
    concat($1,$2) as callsign,
    concat(ai.iata_prefix, (SELECT callsign FROM flightroute_callsign_inner WHERE flightroute_callsign_inner_id = iata_prefix_id)) AS callsign_iata,
    concat(ai.icao_prefix, (SELECT callsign FROM flightroute_callsign_inner WHERE flightroute_callsign_inner_id = icao_prefix_id)) AS callsign_icao,

    ai.airline_name,
    ai.airline_callsign,
    ai.iata_prefix AS airline_iata,
    ai.icao_prefix AS airline_icao,
    (SELECT country_name FROM COUNTRY where country_id = ai.country_id) as airline_country_name,
    (SELECT country_iso_name FROM COUNTRY where country_id = ai.country_id) as airline_country_iso_name,

    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apo.airport_id ) AS origin_airport_longitude,
          
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apm.airport_id ) AS midpoint_airport_longitude,
            
    ( SELECT country_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_name,
    ( SELECT country_iso_name FROM airport oa JOIN country USING(country_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_country_iso_name,
    ( SELECT municipality FROM airport oa JOIN airport_municipality USING(airport_municipality_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_municipality,
    ( SELECT icao_code FROM airport oa JOIN airport_icao_code USING(airport_icao_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_icao_code,
    ( SELECT iata_code FROM airport oa JOIN airport_iata_code USING(airport_iata_code_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_iata_code,
    ( SELECT name FROM airport oa JOIN airport_name USING(airport_name_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_name,
    ( SELECT elevation FROM airport oa JOIN airport_elevation USING(airport_elevation_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_elevation,
    ( SELECT latitude FROM airport oa JOIN airport_latitude USING(airport_latitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_latitude,
    ( SELECT longitude FROM airport oa JOIN airport_longitude USING(airport_longitude_id) WHERE oa.airport_id = apd.airport_id ) AS destination_airport_longitude

FROM flightroute fl
LEFT JOIN flightroute_callsign flc USING(flightroute_callsign_id)
LEFT JOIN flightroute_callsign_inner fci ON fci.flightroute_callsign_inner_id = flc.callsign_id
LEFT JOIN airline ai USING(airline_id)
LEFT JOIN airport apo ON apo.airport_id = fl.airport_origin_id
LEFT JOIN airport apm ON apm.airport_id = fl.airport_midpoint_id
LEFT JOIN airport apd ON apd.airport_id = fl.airport_destination_id

WHERE 
    flc.airline_id = (SELECT DISTINCT(ai.airline_id) FROM flightroute_callsign flc LEFT JOIN airline USING(airline_id) WHERE ai.iata_prefix = $1 LIMIT 1)
AND
    flc.icao_prefix_id = (SELECT flightroute_callsign_inner_id FROM flightroute_callsign_inner WHERE callsign = $2 LIMIT 1)"
    }

    /// Update self, with new origin or destination, or both, and clear cache
    pub async fn update(
        &self,
        postgres: &PgPool,
        redis: &mut Connection,
        origin: ModelAirport,
        destination: ModelAirport,
    ) -> Result<(), AppError> {
        let query = "UPDATE flightroute SET airport_origin_id = $1, airport_destination_id = $2 WHERE flightroute_id = $3";

        sqlx::query(query)
            .bind(origin.airport_id)
            .bind(destination.airport_id)
            .bind(self.flightroute_id)
            .execute(postgres)
            .await?;

        if let Some(iata) = self.callsign_iata.as_ref() {
            redis.del(format!("callsign::{iata}")).await?;
        }

        if let Some(icao) = self.callsign_icao.as_ref() {
            redis.del(format!("callsign::{icao}")).await?;
        }

        Ok(())
    }
}
