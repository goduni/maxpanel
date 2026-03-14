// TODO: For production deployments behind a reverse proxy, extract the
// real client IP from X-Forwarded-For or similar headers. This requires configuring
// the trusted proxy depth to prevent IP spoofing. Currently rate limiting uses
// application-level keys (e.g. email) rather than IP-based keys.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;

/// In-process token-bucket rate limiter using DashMap.
/// NOTE: State is not shared across instances. In a horizontally-scaled
/// deployment, each instance maintains independent counters. Consider
/// Redis-backed rate limiting for multi-instance deployments.
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
    last_access: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
            last_access: Instant::now(),
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
        self.last_access = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

impl RateLimiter {
    pub fn new(cancel: CancellationToken) -> Self {
        let limiter = Self {
            buckets: Arc::new(DashMap::new()),
        };

        // Spawn cleanup task that respects cancellation
        let buckets = Arc::clone(&limiter.buckets);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        tracing::debug!("Rate limiter cleanup task shutting down");
                        return;
                    }
                    _ = interval.tick() => {
                        let cutoff = Instant::now() - Duration::from_secs(120);
                        buckets.retain(|_, v| v.last_access > cutoff);
                    }
                }
            }
        });

        limiter
    }

    pub fn check(&self, key: &str, max_tokens: f64, refill_rate: f64) -> bool {
        // Fast path: check existing bucket without allocation
        if let Some(mut bucket) = self.buckets.get_mut(key) {
            return bucket.try_consume();
        }
        // Slow path: allocate key and insert new bucket
        let mut entry = self.buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket::new(max_tokens, refill_rate));
        entry.try_consume()
    }
}
