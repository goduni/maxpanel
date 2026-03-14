use std::time::Duration;

use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

/// Background worker that deletes expired refresh tokens daily.
/// Prevents the refresh_tokens table from growing unboundedly.
pub async fn run_token_cleanup(pool: PgPool, cancel: CancellationToken) {
    // Run immediately at startup
    match sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < now()")
        .execute(&pool)
        .await
    {
        Ok(result) => {
            let count = result.rows_affected();
            if count > 0 {
                tracing::info!(count, "Expired refresh tokens cleaned up at startup");
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to clean up expired refresh tokens at startup");
        }
    }

    let mut interval = tokio::time::interval(Duration::from_secs(86400));
    interval.tick().await; // skip immediate tick (already ran above)

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Token cleanup worker shutting down");
                return;
            }
            _ = interval.tick() => {
                match sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < now()")
                    .execute(&pool)
                    .await
                {
                    Ok(result) => {
                        let count = result.rows_affected();
                        if count > 0 {
                            tracing::info!(count, "Expired refresh tokens cleaned up");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to clean up expired refresh tokens");
                    }
                }
            }
        }
    }
}
