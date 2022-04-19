mod auth;
mod cache;
mod ratelimit;
mod web;

fn main() {
    env_logger::init();
    auth::init_db().expect("Failed to initialize database");
    web::start();
}
