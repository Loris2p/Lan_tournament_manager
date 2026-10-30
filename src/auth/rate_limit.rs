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

    /// Vérifie si l'adresse IP a dépassé la limite autorisée.
    /// Renvoie `true` si la requête est acceptée et enregistrée, `false` si le quota est dépassé.
    pub fn check(&self, ip: &str) -> bool {
        let mut map = match self.attempts.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = Instant::now();
        let list = map.entry(ip.to_string()).or_default();

        // Conserve uniquement les tentatives dans la fenêtre temporelle
        list.retain(|&t| now.duration_since(t) < self.window);

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
