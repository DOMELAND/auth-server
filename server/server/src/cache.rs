use auth_common::AuthToken;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct TimedCacheEntry {
    pub timestamp: Instant,
    pub data: Uuid,
}

fn work_clean(map: Arc<Mutex<HashMap<AuthToken, TimedCacheEntry>>>) {
    loop {
        thread::sleep(Duration::from_secs(60));

        map
            .lock()
            // Panic and restart if the authtoken cache is poisoned which should never happen.
            .expect("AuthToken cache has been poisoned. Panicking to restart.")
            .retain(|_, v| v.timestamp.elapsed() < Duration::from_secs(15));
    }
}

pub struct TimedCache {
    inner: Arc<Mutex<HashMap<AuthToken, TimedCacheEntry>>>,
}

impl TimedCache {
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(HashMap::new()));
        {
            let inner = inner.clone();
            thread::spawn(|| work_clean(inner));
        }
        Self { inner }
    }

    pub fn insert(&self, k: AuthToken, v: Uuid) {
        self
            .inner
            .lock()
            // Panic and restart if the authtoken cache is poisoned which should never happen.
            .expect("AuthToken cache has been poisoned. Panicking to restart.")
            .insert(
            k,
            TimedCacheEntry {
                timestamp: Instant::now(),
                data: v,
            },
        );
    }

    pub fn run(&self, k: &AuthToken, f: impl FnOnce(Option<&mut TimedCacheEntry>) -> bool) {
        let mut inner = self
            .inner
            .lock()
            // Panic and restart if the authtoken cache is poisoned which should never happen.
            .expect("AuthToken cache has been poisoned. Panicking to restart.");

        let v = inner.get_mut(k);
        if !f(v) {
            inner.remove(&k);
        }
    }
}
