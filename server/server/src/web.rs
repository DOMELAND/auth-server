use crate::auth::{self, AuthError};
use crate::ratelimit::RateLimiter;
use auth_common::{
    RegisterPayload, SignInPayload, SignInResponse, UsernameLookupPayload, UsernameLookupResponse,
    UuidLookupPayload, UuidLookupResponse, ValidityCheckPayload, ValidityCheckResponse,
};
use lazy_static::lazy_static;
use log::*;
use rouille::{start_server, Request, Response};
use std::net::IpAddr;

lazy_static! {
    static ref RATELIMITER: RateLimiter = RateLimiter::new();
}

fn legal_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || ['-', '_'].contains(&c)
}

fn legal_ethaddr(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn verify_username(username: &str) -> Result<(), AuthError> {
    if !(3..=32).contains(&username.len()) {
        Err(AuthError::InvalidRequest(
            "Username must be between 3 and 32 characters inclusive.".into(),
        ))
    } else if !username.chars().all(legal_char) {
        Err(AuthError::InvalidRequest(
            "Illegal character in username.".into(),
        ))
    } else {
        Ok(())
    }
}

// add new verify fn -max
fn verify_ethaddr(ethaddr: &str) -> Result<(), AuthError> {
    //Eth address save with the hex prefix ("0x"), so it's 42 characters length.
    if !(42 == ethaddr.len()) {   
        Err(AuthError::InvalidEthAddr(
            "Eth address must be between 42 characters with the hex prefix '0x'.".into(),
        ))
    } else if !ethaddr.chars().all(legal_ethaddr) {
        Err(AuthError::InvalidEthAddr(
            "Illegal character in Ethrum address.".into(),
        ))
    } else {
        Ok(())
    }
}

fn ratelimit(
    req: &Request,
    f: fn(&Request) -> Result<Response, AuthError>,
) -> Result<Response, AuthError> {
    if RATELIMITER.check(remote(req)) {
        f(req)
    } else {
        Err(AuthError::RateLimit)
    }
}

fn remote(req: &Request) -> IpAddr {
    req.header("X-Real-IP")
        .map(|ip| ip.parse().unwrap_or(req.remote_addr().ip()))
        .unwrap_or(req.remote_addr().ip())
}

fn ping(req: &Request) -> Response {
    Response::text(format!("Ping! {}", remote(req).to_string()))
}

fn username_to_uuid(req: &Request) -> Result<Response, AuthError> {
    let body = req.data().unwrap();
    let payload: UuidLookupPayload = serde_json::from_reader(body)?;
    let uuid = auth::username_to_uuid(&payload.username)?;
    let response = UuidLookupResponse { uuid };
    Ok(Response::json(&response))
}

fn uuid_to_username(req: &Request) -> Result<Response, AuthError> {
    let body = req.data().unwrap();
    let payload: UsernameLookupPayload = serde_json::from_reader(body)?;
    let username = auth::uuid_to_username(&payload.uuid)?;
    let response = UsernameLookupResponse { username };
    Ok(Response::json(&response))
}

fn register(req: &Request) -> Result<Response, AuthError> {
    let body = req.data().unwrap();
    let payload: RegisterPayload = serde_json::from_reader(body)?;
    verify_username(&payload.username)?;
    verify_ethaddr(&payload.ethaddr)?;   // new verify  -max
    auth::register(&payload.username, &payload.password,&payload.ethaddr)?;
    Ok(Response::text("Ok"))
}

fn generate_token(req: &Request) -> Result<Response, AuthError> {
    let body = req.data().unwrap();
    let payload: SignInPayload = serde_json::from_reader(body)?;
    verify_username(&payload.username)?;
    let token = auth::generate_token(&payload.username, &payload.password)?;
    let response = SignInResponse { token };
    Ok(Response::json(&response))
}

fn verify(req: &Request) -> Result<Response, AuthError> {
    let body = req.data().unwrap();
    let payload: ValidityCheckPayload = serde_json::from_reader(body)?;
    let uuid = auth::verify(payload.token)?;
    let response = ValidityCheckResponse { uuid };
    Ok(Response::json(&response))
}

pub fn start() {
    let addr = "0.0.0.0:19253";
    debug!("Starting webserver on {}", addr);

    start_server(addr, move |request| {
        debug!("[{}] -> {}", remote(request), request.url());

        let path = request.raw_url().split('?').next().unwrap();

        let response = match (request.method(), path) {
            ("GET", "/ping") => ping(request),
            ("POST", path) => {
                let result = match path {
                    "/username_to_uuid" => username_to_uuid(request),
                    "/uuid_to_username" => uuid_to_username(request),
                    "/register" => ratelimit(request, register),
                    "/generate_token" => ratelimit(request, generate_token),
                    "/verify" => verify(request),
                    _ => Ok(Response::empty_404()),
                };

                match result {
                    Ok(response) => response,
                    Err(err) => {
                        info!("[{}:{}] rejected: {}", remote(request), path, err);

                        Response::text(format!("{}", err)).with_status_code(err.status_code())
                    }
                }
            }
            _ => Response::empty_404(),
        };

        response.with_unique_header("Access-Control-Allow-Origin", "*")
    });
}