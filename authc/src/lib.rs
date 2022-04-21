use argon2::Config;
pub use auth_common::AuthToken;
use auth_common::{
    RegisterPayload, SignInPayload, SignInResponse, UsernameLookupPayload, UsernameLookupResponse,
    UuidLookupPayload, UuidLookupResponse, ValidityCheckPayload, ValidityCheckResponse,
};
use reqwest::{IntoUrl, Url};
pub use uuid::Uuid;

fn net_prehash(password: &str) -> String {
    let salt = fxhash::hash64(password);
    let config = Config::default();
    let bytes = argon2::hash_raw(password.as_bytes(), &salt.to_le_bytes(), &config).unwrap();
    hex::encode(&bytes)
}

#[derive(Debug)]
pub enum AuthClientError {
    // Server did not return 200-299 StatusCode.
    ServerError(u16, String),
    RequestError(reqwest::Error),
    InvalidUrl(url::ParseError),
}
pub struct AuthClient {
    client: reqwest::blocking::Client,
    provider: Url,
}

impl AuthClient {
    pub fn new<T: IntoUrl>(provider: T) -> Result<Self, AuthClientError> {
        Ok(Self {
            client: reqwest::blocking::Client::new(),
            provider: provider.into_url()?,
        })
    }

    pub fn register(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
        ethaddr:  impl AsRef<str>,    // new add -max
    ) -> Result<(), AuthClientError> {
        let data = RegisterPayload {
            username: username.as_ref().to_owned(),
            password: net_prehash(password.as_ref()),
            ethaddr: ethaddr.as_ref().to_owned(),
        };
        let ep = self.provider.join("register")?;
        self.client.post(ep).json(&data).send()?;
        println!("register posted request");
        Ok(())
    }

    pub fn username_to_uuid(
        &self,
        username: impl AsRef<str>,
    ) -> Result<Uuid, AuthClientError> {
        let data = UuidLookupPayload {
            username: username.as_ref().to_owned(),
        };
        let ep = self.provider.join("username_to_uuid")?;
        let resp = self.client.post(ep).json(&data).send()?;

        Ok(handle_response::<UuidLookupResponse>(resp)?.uuid)
    }

    pub fn uuid_to_username(&self, uuid: Uuid) -> Result<String, AuthClientError> {
        let data = UsernameLookupPayload { uuid };
        let ep = self.provider.join("uuid_to_username")?;
        let resp = self.client.post(ep).json(&data).send()?;

        Ok(handle_response::<UsernameLookupResponse>(resp)?
            .username)
    }

    pub fn sign_in(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<AuthToken, AuthClientError> {
        let data = SignInPayload {
            username: username.as_ref().to_owned(),
            password: net_prehash(password.as_ref()),
        };

        let ep = self.provider.join("generate_token")?;
        let resp = self.client.post(ep).json(&data).send()?;

        Ok(handle_response::<SignInResponse>(resp)?.token)
    }

    pub fn validate(&self, token: AuthToken) -> Result<Uuid, AuthClientError> {
        let data = ValidityCheckPayload { token };

        let ep = self.provider.join("verify")?;
        let resp = self.client.post(ep).json(&data).send()?;

        Ok(handle_response::<ValidityCheckResponse>(resp)?.uuid)
    }
}

/// If response code isn't a success it will return an error with the response code and plain text body.
///
/// Otherwise will deserialize the json based on given type (through turbofish notation)
fn handle_response<T>(resp: reqwest::blocking::Response) -> Result<T, AuthClientError>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    if resp.status().is_success() {
        Ok(resp.json::<T>()?)
    } else {
        Err(AuthClientError::ServerError(
            resp.status().as_u16(),
            resp.text()?,
        ))
    }
}

impl std::fmt::Display for AuthClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            AuthClientError::ServerError(code, text) => {
                write!(f, "Auth Server returned {} with: {}", code, text)
            }
            AuthClientError::RequestError(text) => write!(f, "Request failed with: {}", text),
            AuthClientError::InvalidUrl(e) => {
                write!(f, "Got invalid url to make auth requests to: {}", e)
            }
        }
    }
}

impl From<url::ParseError> for AuthClientError {
    fn from(err: url::ParseError) -> Self {
        AuthClientError::InvalidUrl(err)
    }
}

impl From<reqwest::Error> for AuthClientError {
    fn from(err: reqwest::Error) -> Self {
        AuthClientError::RequestError(err)
    }
}
