use std::env;
use jsonwebtoken::{EncodingKey, DecodingKey};
use url::Url;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_min_connections: u32,
    pub database_acquire_timeout_secs: u64,
    pub jwt_secret: String,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub jwt_access_ttl_secs: u64,
    pub refresh_token_ttl_days: u64,
    pub refresh_token_hmac_secret: String,
    pub invite_token_hmac_secret: String,
    pub bot_api_key_hmac_secret: String,
    pub bot_token_encryption_key: [u8; 32],
    pub webhook_base_url: String,
    pub cors_allowed_origins: Vec<String>,
    pub app_env: AppEnv,
    pub listen_addr: String,
    pub max_api_base_url: String,
    pub max_api_host: String,
    pub polling_concurrency: usize,
    pub invite_ttl_days: i64,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"[REDACTED]")
            .field("database_max_connections", &self.database_max_connections)
            .field("database_min_connections", &self.database_min_connections)
            .field("database_acquire_timeout_secs", &self.database_acquire_timeout_secs)
            .field("jwt_secret", &"[REDACTED]")
            .field("jwt_encoding_key", &"[REDACTED]")
            .field("jwt_decoding_key", &"[REDACTED]")
            .field("jwt_access_ttl_secs", &self.jwt_access_ttl_secs)
            .field("refresh_token_ttl_days", &self.refresh_token_ttl_days)
            .field("refresh_token_hmac_secret", &"[REDACTED]")
            .field("invite_token_hmac_secret", &"[REDACTED]")
            .field("bot_api_key_hmac_secret", &"[REDACTED]")
            .field("bot_token_encryption_key", &"[REDACTED]")
            .field("webhook_base_url", &self.webhook_base_url)
            .field("cors_allowed_origins", &self.cors_allowed_origins)
            .field("app_env", &self.app_env)
            .field("listen_addr", &self.listen_addr)
            .field("max_api_base_url", &self.max_api_base_url)
            .field("max_api_host", &self.max_api_host)
            .field("polling_concurrency", &self.polling_concurrency)
            .field("invite_ttl_days", &self.invite_ttl_days)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppEnv {
    Development,
    Production,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Try .env in current dir first, then parent (monorepo root)
        if dotenvy::dotenv().is_err() {
            dotenvy::from_filename("../.env").ok();
        }

        let app_env = match env::var("APP_ENV").unwrap_or_else(|_| "development".into()).as_str() {
            "production" => AppEnv::Production,
            _ => AppEnv::Development,
        };

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?;
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 bytes");
        }
        let jwt_encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        let jwt_decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        let refresh_token_hmac_secret = env::var("REFRESH_TOKEN_HMAC_SECRET")
            .map_err(|_| anyhow::anyhow!("REFRESH_TOKEN_HMAC_SECRET is required"))?;
        if refresh_token_hmac_secret.len() < 32 {
            anyhow::bail!("REFRESH_TOKEN_HMAC_SECRET must be at least 32 bytes");
        }

        let invite_token_hmac_secret = env::var("INVITE_TOKEN_HMAC_SECRET")
            .map_err(|_| anyhow::anyhow!("INVITE_TOKEN_HMAC_SECRET is required"))?;
        if invite_token_hmac_secret.len() < 32 {
            anyhow::bail!("INVITE_TOKEN_HMAC_SECRET must be at least 32 bytes");
        }

        let bot_api_key_hmac_secret = env::var("BOT_API_KEY_HMAC_SECRET")
            .map_err(|_| anyhow::anyhow!("BOT_API_KEY_HMAC_SECRET is required"))?;
        if bot_api_key_hmac_secret.len() < 32 {
            anyhow::bail!("BOT_API_KEY_HMAC_SECRET must be at least 32 bytes");
        }

        let encryption_key_hex = env::var("BOT_TOKEN_ENCRYPTION_KEY")
            .map_err(|_| anyhow::anyhow!("BOT_TOKEN_ENCRYPTION_KEY is required"))?;
        let encryption_key_bytes = crate::utils::hex_decode(&encryption_key_hex)
            .map_err(|_| anyhow::anyhow!("BOT_TOKEN_ENCRYPTION_KEY must be a valid hex-encoded 32-byte key"))?;
        let bot_token_encryption_key: [u8; 32] = encryption_key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("BOT_TOKEN_ENCRYPTION_KEY must be exactly 32 bytes"))?;

        let webhook_base_url = env::var("WEBHOOK_BASE_URL")
            .map_err(|_| anyhow::anyhow!("WEBHOOK_BASE_URL is required"))?;

        if app_env == AppEnv::Production {
            let parsed = url::Url::parse(&webhook_base_url)
                .map_err(|e| anyhow::anyhow!("Invalid WEBHOOK_BASE_URL: {}", e))?;
            if parsed.scheme() != "https" {
                anyhow::bail!("WEBHOOK_BASE_URL must use HTTPS in production");
            }
        }

        let cors_origins_str = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".into());
        let cors_allowed_origins: Vec<String> = cors_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if app_env == AppEnv::Production {
            if cors_allowed_origins.is_empty() {
                anyhow::bail!("CORS_ALLOWED_ORIGINS is required in production");
            }
            if cors_allowed_origins.iter().any(|o| o.contains("localhost")) {
                anyhow::bail!("CORS_ALLOWED_ORIGINS must not contain localhost in production");
            }
            for origin in &cors_allowed_origins {
                if !origin.starts_with("https://") {
                    anyhow::bail!("CORS_ALLOWED_ORIGINS must use HTTPS in production, got: {}", origin);
                }
            }
        }

        let max_api_base_url = env::var("MAX_API_BASE_URL")
            .unwrap_or_else(|_| "https://platform-api.max.ru".into());
        // Validate MAX_API_BASE_URL
        let parsed_url = Url::parse(&max_api_base_url)
            .map_err(|e| anyhow::anyhow!("Invalid MAX_API_BASE_URL: {}", e))?;
        if app_env == AppEnv::Production {
            let allowed_hosts = ["platform-api.max.ru"];
            if !allowed_hosts.contains(&parsed_url.host_str().unwrap_or_default()) {
                anyhow::bail!("MAX_API_BASE_URL host must be one of: {:?}", allowed_hosts);
            }
        }

        let max_api_host = parsed_url
            .host_str()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let database_max_connections: u32 = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "50".into())
            .parse()?;
        let database_min_connections: u32 = env::var("DATABASE_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "10".into())
            .parse()?;
        if database_min_connections > database_max_connections {
            anyhow::bail!("DATABASE_MIN_CONNECTIONS must not exceed DATABASE_MAX_CONNECTIONS");
        }

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            database_max_connections,
            database_min_connections,
            database_acquire_timeout_secs: env::var("DATABASE_ACQUIRE_TIMEOUT_SECS")
                .unwrap_or_else(|_| "5".into())
                .parse()?,
            jwt_secret,
            jwt_encoding_key,
            jwt_decoding_key,
            jwt_access_ttl_secs: env::var("JWT_ACCESS_TTL_SECS")
                .unwrap_or_else(|_| "900".into())
                .parse()?,
            refresh_token_ttl_days: env::var("REFRESH_TOKEN_TTL_DAYS")
                .unwrap_or_else(|_| "30".into())
                .parse()?,
            refresh_token_hmac_secret,
            invite_token_hmac_secret,
            bot_api_key_hmac_secret,
            bot_token_encryption_key,
            webhook_base_url,
            cors_allowed_origins,
            app_env,
            listen_addr: env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into()),
            max_api_base_url,
            max_api_host,
            polling_concurrency: env::var("POLLING_CONCURRENCY")
                .unwrap_or_else(|_| "50".into())
                .parse()?,
            invite_ttl_days: env::var("INVITE_TTL_DAYS")
                .unwrap_or_else(|_| "7".into())
                .parse()?,
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env == AppEnv::Production
    }
}

