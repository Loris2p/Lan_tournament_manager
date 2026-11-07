use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RateLimiter {
    attempts: Mutex<HashMap<String, Vec<Instant>>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window: Duration) -> Self {
        Self {
            attempts: Mutex::new(HashMap::new()),
            max_attempts,
            window,
        }
    }

    pub fn check(&self, ip: &str) -> bool {
        let mut map = self.attempts.lock(). unwrap_or_else(|poisoned| poisoned.into_inner());
        let now = Instant::now();
        let list = map.entry(ip.to_string()).or_default();

        // Conserve uniquement les tentatives dans la fenêtre temporelle
        list.retain(|&t| now.saturating_duration_since(t) < self.window);

        if list.len() >= self.max_attempts {
            false
        } else {
            list.push(now);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));
        let ip = "192.168.1.50";

        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        // 4e tentative dans la même fenêtre -> rejeté
        assert!(!limiter.check(ip));

        // Une autre IP ne doit pas être bloquée
        assert!(limiter.check("192.168.1.51"));
    }
}
