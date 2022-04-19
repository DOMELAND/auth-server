use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX: usize = 60;
const TIMEOUT: Duration = Duration::from_secs(60 * 10);

pub struct RateLimiter {
    limits: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            limits: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, addr: IpAddr) -> bool {
        if addr.is_loopback() {
            return true;
        }

        let mut limits = self
            .limits
            .lock()
            // Panic and restart if the authtoken cache is poisoned which should never happen.
            .expect("AuthToken cache has been poisoned. Panicking to restart.");

        let v = limits.entry(addr).or_default();
        v.push(Instant::now());
        v.retain(|t| t.elapsed() < TIMEOUT);
        v.len() <= MAX
    }
}
