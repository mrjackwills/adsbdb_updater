use std::fmt;

use crate::{
    app_error::AppError,
    n_number::{n_number_to_mode_s, ALLCHARS},
};

/// Check that a given char is 0-9, a-END, will lowercase everything
fn valid_char(c: char, end: char) -> bool {
    c.is_ascii_digit() || ('a'..=end).contains(&c.to_ascii_lowercase())
}

// This should take impl to string and result as Self
pub trait Validate {
    fn validate(x: &str) -> Result<Self, AppError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeS(String);

impl fmt::Display for ModeS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Validate for ModeS {
    /// Make sure that input is an uppercase valid mode_s string, validity is [a-f]{6}
    fn validate(input: &str) -> Result<Self, AppError> {
        let input = input.to_uppercase();
        if input.len() == 6 && input.chars().all(|c| valid_char(c, 'f')) {
            Ok(Self(input))
        } else {
            Err(AppError::ModeS(input))
        }
    }
}

// Split this into an enum, Icao, Iata, Other
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Callsign {
    // Could put optional ModelAirline in here?
    Icao((String, String)),
    Iata((String, String)),
    Other(String),
}

impl fmt::Display for Callsign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Icao(x) | Self::Iata(x) => write!(f, "{}{}", x.0, x.1),
            Self::Other(x) => write!(f, "{x}"),
        }
    }
}

impl Validate for Callsign {
    // Make sure that input is a valid callsign String, validity is [a-z]{4-8}
    // output into Callsign Enum
    fn validate(input: &str) -> Result<Self, AppError> {
        let input = input.to_uppercase();
        if (4..=8).contains(&input.len()) && input.chars().all(|c| valid_char(c, 'z')) {
            let icao = input.split_at(3);
            let iata = input.split_at(2);
            if icao
                .0
                .chars()
                .all(|c: char| c.to_ascii_lowercase().is_ascii_lowercase())
            {
                Ok(Self::Icao((icao.0.to_owned(), icao.1.to_owned())))
            } else if iata.0.chars().all(|c| valid_char(c, 'z')) {
                if let Ok(n_number) = NNumber::validate(&input) {
                    if n_number_to_mode_s(&n_number).is_ok() {
                        return Ok(Self::Other(input));
                    }
                }
                Ok(Self::Iata((iata.0.to_owned(), iata.1.to_owned())))
            } else {
                Ok(Self::Other(input))
            }
        } else {
            Err(AppError::Callsign(input))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NNumber(String);

impl fmt::Display for NNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Validate for NNumber {
    /// Make sure that input is an uppercase valid n_number string, validity is N[0-9 a-z (but not I or O)]{1-5}
    fn validate(input: &str) -> Result<Self, AppError> {
        let input = input.to_uppercase();
        if input.starts_with('N')
            && (2..=6).contains(&input.chars().count())
            && input.chars().all(|x| ALLCHARS.contains(x))
        {
            Ok(Self(input))
        } else {
            Err(AppError::NNumber(input))
        }
    }
}
