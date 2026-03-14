use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::Config;
use crate::middleware::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub video_url_cache: Arc<DashMap<(Uuid, String), (String, Instant)>>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config, cancel: CancellationToken) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .pool_max_idle_per_host(20)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            db,
            config: Arc::new(config),
            http_client,
            rate_limiter: RateLimiter::new(cancel),
            video_url_cache: Arc::new(DashMap::new()),
        }
    }
}
