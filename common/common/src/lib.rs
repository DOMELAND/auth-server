#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AuthToken {
    pub unique: u64,
}

impl std::str::FromStr for AuthToken {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u64>() {
            Ok(s) => Ok(AuthToken { unique: s }),
            Err(e) => Err(e),
        }
    }
}

impl AuthToken {
    pub fn generate() -> Self {
        Self {
            unique: rand::random(),
        }
    }

    pub fn serialize(&self) -> String {
        self.unique.to_string()
    }

    pub fn deserialize(s: &str) -> Self {
        let n = s.parse().unwrap();
        Self { unique: n }
    }
}

// add ethaddr field.  -max.lee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub ethaddr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignInResponse {
    pub token: AuthToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidityCheckPayload {
    pub token: AuthToken,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ValidityCheckResponse {
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UuidLookupPayload {
    pub username: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UuidLookupResponse {
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameLookupPayload {
    pub uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameLookupResponse {
    pub username: String,
}
