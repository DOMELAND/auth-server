use crate::cache::TimedCache;
use argon2::Error as HashError;
use auth_common::AuthToken;
use lazy_static::lazy_static;
use rusqlite::{params, Connection, Error as DbError, NO_PARAMS};
use serde_json::Error as JsonError;
use std::error::Error;
use std::fmt;
use uuid::Uuid;
use std::{env, path::PathBuf};

lazy_static! {
    static ref TOKENS: TimedCache = TimedCache::new();
}

fn apply_db_dir_override(db_dir: &str) -> String {
    if let Some(val) = env::var_os("AUTH_DB_DIR") {
        let path = PathBuf::from(val);
        if path.exists() || path.parent().map(|x| x.exists()).unwrap_or(false) {
            // Only allow paths with valid unicode characters
            match path.to_str() {
                Some(path) => return path.to_owned(),
                None => {},
            }
        }
        log::warn!("AUTH_DB_DIR is an invalid path.");
    }
    db_dir.to_string()
}

fn db() -> Result<Connection, AuthError> {
    let db_dir = &apply_db_dir_override("/opt/veloren-auth/data/auth.db");
    Ok(Connection::open(db_dir)?)
}

fn salt() -> [u8; 16] {
    rand::random::<u128>().to_le_bytes()
}

fn decapitalize(string: &str) -> String {
    string.chars().flat_map(char::to_lowercase).collect()
}

#[derive(Debug)]
pub enum AuthError {
    UserExists,
    UserDoesNotExist,
    EthDoesNotExist,
    InvalidLogin,
    InvalidToken,
    Db(DbError),
    Hash(HashError),
    Json(JsonError),
    InvalidRequest(String),
    InvalidEthAddr(String),
    RateLimit,
}

impl AuthError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::UserExists => 400,
            Self::UserDoesNotExist => 400,
            Self::EthDoesNotExist => 400,
            Self::InvalidLogin => 400,
            Self::InvalidToken => 400,
            Self::Db(_) => 500,
            Self::Hash(_) => 500,
            Self::Json(_) => 400,
            Self::InvalidRequest(_) => 400,
            Self::InvalidEthAddr(_) => 400,
            Self::RateLimit => 429,
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::UserExists => "That username is already taken.".into(),
                Self::UserDoesNotExist => "That user does not exist.".into(),
                Self::EthDoesNotExist => "That ethereum address does not exist.".into(),
                Self::InvalidLogin =>
                    "The username + password + Eth_addr combination was incorrect or the user does not exist."
                        .into(),
                Self::InvalidToken => "The given token is invalid.".into(),
                Self::Db(err) => format!("Database error: {}", err),
                Self::Hash(err) => format!("Error securely storing password: {}", err),
                Self::Json(err) => format!("Error decoding JSON: {}", err),
                Self::InvalidRequest(s) =>
                    format!("The request was invalid in some form. Reason: {}", s),
                Self::InvalidEthAddr(s) =>
                    format!("The given eth addr is invalid: {}", s),
                Self::RateLimit => "You are sending too many requests. Please slow down.".into(),
            }
        )
    }
}

impl Error for AuthError {}

impl From<DbError> for AuthError {
    fn from(err: DbError) -> Self {
        Self::Db(err)
    }
}

impl From<HashError> for AuthError {
    fn from(err: HashError) -> Self {
        Self::Hash(err)
    }
}

impl From<JsonError> for AuthError {
    fn from(err: JsonError) -> Self {
        Self::Json(err)
    }
}

pub fn init_db() -> Result<(), AuthError> {    //add ethaddr filed in users tables  -max
    db()?.execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            uuid TEXT NOT NULL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            display_username TEXT NOT NULL UNIQUE,
            ethaddr TEXT NOT NULL UNIQUE,
            pwhash TEXT NOT NULL
        )
    ",
        NO_PARAMS,
    )?;
    Ok(())
}

fn user_exists(username: &str) -> Result<bool, AuthError> {
    let db = db()?;
    let mut stmt = db.prepare("SELECT uuid FROM users WHERE username == ?1")?;
    Ok(stmt.exists(params![username])?)
}

pub fn username_to_uuid(username_unfiltered: &str) -> Result<Uuid, AuthError> {
    let username = decapitalize(username_unfiltered);
    let db = db()?;
    let mut stmt = db.prepare_cached("SELECT uuid FROM users WHERE username == ?1")?;
    let result = stmt
        .query_map(params![&username], |row| row.get::<_, String>(0))?
        .filter_map(|s| s.ok())
        .filter_map(|s| Uuid::parse_str(&s).ok())
        .next()
        .ok_or(AuthError::UserDoesNotExist);
    result
}

pub fn uuid_to_username(uuid: &Uuid) -> Result<String, AuthError> {
    let db = db()?;
    let uuid = uuid.to_simple().to_string();
    let mut stmt = db.prepare_cached("SELECT display_username FROM users WHERE uuid == ?1")?;
    let result = stmt
        .query_map(params![uuid], |row| row.get::<_, String>(0))?
        .filter_map(|s| s.ok())
        .next()
        .ok_or(AuthError::UserDoesNotExist);
    result
}


pub fn eth_to_uuid(ethaddr_unfiltered: &str) -> Result<Uuid, AuthError> {
    let ethaddr = decapitalize(ethaddr_unfiltered);
    let db = db()?;
    let mut stmt = db.prepare_cached("SELECT uuid FROM users WHERE ethaddr == ?1")?;
    let result = stmt
        .query_map(params![&ethaddr], |row| row.get::<_, String>(0))?
        .filter_map(|s| s.ok())
        .next()
        .ok_or(AuthError::EthDoesNotExist);
    result
}


pub fn eth_to_username(ethaddr_unfiltered: &str) -> Result<String, AuthError> {
    let ethaddr = decapitalize(ethaddr_unfiltered);
    let db = db()?;
    let mut stmt = db.prepare_cached("SELECT display_username FROM users WHERE ethaddr == ?1")?;
    let result = stmt
        .query_map(params![&ethaddr], |row| row.get::<_, String>(0))?
        .filter_map(|s| s.ok())
        .next()
        .ok_or(AuthError::EthDoesNotExist);
    result
}

// add new parameter ethaddr -max
pub fn register(username_unfiltered: &str, password: &str, ethaddr_unfiltered: &str) -> Result<(), AuthError> {
    let username = decapitalize(username_unfiltered);
    let ethaddr = decapitalize(ethaddr_unfiltered);
    if user_exists(&username)? {
        println!("user exists");
        return Err(AuthError::UserExists);
    }
    
    let uuid = Uuid::new_v4().to_simple().to_string();
    let hconfig = argon2::Config::default();
    let pwhash = argon2::hash_encoded(password.as_bytes(), &salt(), &hconfig)?;
    println!("user go");
    db()?.execute(
        "INSERT INTO users (uuid, username, display_username, ethaddr, pwhash) VALUES(?1, ?2, ?3, ?4, ?5)",
        params![uuid, &username, username_unfiltered, ethaddr, pwhash],
    )?;
    println!("user go2");
    Ok(())
}

/// Checks if the password is correct and that the user exists.
fn is_valid(username: &str, password: &str) -> Result<bool, AuthError> {
    let db = db()?;
    let mut stmt = db.prepare_cached("SELECT pwhash FROM users WHERE username == ?1")?;
    let result = stmt
        .query_map(params![&username], |row| row.get::<_, String>(0))?
        .filter_map(|s| s.ok())
        .filter_map(|correct| argon2::verify_encoded(&correct, password.as_bytes()).ok())
        .next()
        .ok_or(AuthError::InvalidLogin);
    result
}

pub fn generate_token(username_unfiltered: &str, password: &str) -> Result<AuthToken, AuthError> {
    let username = decapitalize(username_unfiltered);
    if !is_valid(&username, password)? {
        return Err(AuthError::InvalidLogin);
    }

    let uuid = username_to_uuid(&username)?;
    let token = AuthToken::generate();
    TOKENS.insert(token, uuid);
    Ok(token)
}

pub fn verify(token: AuthToken) -> Result<Uuid, AuthError> {
    let mut uuid = None;
    TOKENS.run(&token, |entry| {
        uuid = entry.map(|e| e.data.clone());
        false
    });
    uuid.ok_or(AuthError::InvalidToken)
}
