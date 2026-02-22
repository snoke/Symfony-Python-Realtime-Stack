pub(crate) struct RateLimiter {
    rate_per_sec: f64,
    burst: f64,
    tokens: f64,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    pub(crate) fn new(rate_per_sec: f64, burst: f64) -> Self {
        let mut limiter = Self {
            rate_per_sec,
            burst,
            tokens: burst,
            last_refill: std::time::Instant::now(),
        };
        if rate_per_sec <= 0.0 {
            limiter.tokens = f64::INFINITY;
        }
        limiter
    }

    pub(crate) fn allow(&mut self) -> bool {
        if self.rate_per_sec <= 0.0 {
            return true;
        }
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        self.tokens = (self.tokens + elapsed * self.rate_per_sec).min(self.burst);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
