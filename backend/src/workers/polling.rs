use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::config::Config;
use crate::db;
use crate::services::{bots as bots_svc, ingestion, max_api};

/// Whether the polling loop should continue or stop.
enum PollOutcome {
    /// Bot returned data successfully; continue polling.
    Continue,
    /// Bot is deleted or deactivated; stop polling.
    Stop,
}

/// Polling supervisor: periodically scans for active polling bots,
/// spawns a task for each one, restarts on failure with backoff.
pub async fn run_polling_supervisor(
    pool: PgPool,
    config: Arc<Config>,
    http_client: reqwest::Client,
    cancel: CancellationToken,
) {
    let semaphore = Arc::new(Semaphore::new(config.polling_concurrency));
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    // In-process dedup map — the primary dedup mechanism
    let active_bots: Arc<DashMap<Uuid, ()>> = Arc::new(DashMap::new());

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Polling supervisor shutting down");
                return;
            }
            _ = interval.tick() => {
                if let Err(e) = spawn_missing_pollers(
                    &pool, &config, &http_client, &semaphore, &cancel, &active_bots,
                ).await {
                    tracing::error!(error = %e, "Failed to refresh polling bots");
                }
            }
        }
    }
}

async fn spawn_missing_pollers(
    pool: &PgPool,
    config: &Arc<Config>,
    http_client: &reqwest::Client,
    semaphore: &Arc<Semaphore>,
    cancel: &CancellationToken,
    active_bots: &Arc<DashMap<Uuid, ()>>,
) -> Result<(), anyhow::Error> {
    let bot_ids = db::bots::list_active_polling_ids(pool).await?;

    for bot_id in bot_ids {
        if active_bots.contains_key(&bot_id) {
            continue; // already polling
        }

        active_bots.insert(bot_id, ());
        tracing::info!(bot_id = %bot_id, "spawning polling task");

        let pool = pool.clone();
        let config = Arc::clone(config);
        let http_client = http_client.clone();
        let semaphore = Arc::clone(semaphore);
        let cancel = cancel.clone();
        let active_bots = Arc::clone(active_bots);

        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(1);
            let mut cached_token: Option<String> = None;
            let mut cached_context: Option<db::bots::BotPollingContext> = None;
            let mut context_refreshed_at = tokio::time::Instant::now();

            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        active_bots.remove(&bot_id);
                        return;
                    }
                    result = poll_bot(&pool, &config, &http_client, &semaphore, bot_id, &mut cached_token, &mut cached_context, &mut context_refreshed_at) => {
                        match result {
                            Ok(PollOutcome::Continue) => {
                                // Reset backoff, brief pause to avoid tight loop
                                backoff = Duration::from_secs(1);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }
                            Ok(PollOutcome::Stop) => {
                                active_bots.remove(&bot_id);
                                return;
                            }
                            Err(e) => {
                                tracing::error!(bot_id = %bot_id, error = %e, "polling failed, retrying");
                                // Invalidate caches on error — credentials may have changed
                                cached_token = None;
                                cached_context = None;
                                // Add jitter to backoff
                                let jitter: u64 = rand::random_range(0..501);
                                tokio::time::sleep(backoff + Duration::from_millis(jitter)).await;
                                backoff = (backoff * 2).min(Duration::from_secs(60));
                            }
                        }
                    }
                }
            }
        });
    }

    Ok(())
}

/// How often to refresh the cached BotPollingContext from the database.
const CONTEXT_REFRESH_INTERVAL: Duration = Duration::from_secs(60);

async fn poll_bot(
    pool: &PgPool,
    config: &Arc<Config>,
    http_client: &reqwest::Client,
    semaphore: &Arc<Semaphore>,
    bot_id: Uuid,
    cached_token: &mut Option<String>,
    cached_context: &mut Option<db::bots::BotPollingContext>,
    context_refreshed_at: &mut tokio::time::Instant,
) -> Result<PollOutcome, anyhow::Error> {
    // Refresh context from DB if cache is empty or stale (older than 60s)
    if cached_context.is_none() || context_refreshed_at.elapsed() >= CONTEXT_REFRESH_INTERVAL {
        let fresh = db::bots::find_polling_context(pool, bot_id).await?;
        match fresh {
            Some(c) => {
                *cached_context = Some(c);
                *context_refreshed_at = tokio::time::Instant::now();
                *cached_token = None;
            }
            None => return Ok(PollOutcome::Stop),
        }
    }

    let ctx = cached_context.as_ref().unwrap();

    if !ctx.is_active {
        return Ok(PollOutcome::Stop);
    }

    let access_token = match cached_token {
        Some(t) => t.clone(),
        None => {
            let t = bots_svc::decrypt_bot_token_for_polling(config, bot_id, ctx)
                .map_err(|e| anyhow::anyhow!("Failed to decrypt token: {}", e))?;
            *cached_token = Some(t.clone());
            t
        }
    };

    let marker = ctx.polling_marker;

    // Acquire semaphore permit for outbound request
    let _permit = semaphore.acquire().await?;
    let response = max_api::get_updates(http_client, config, &access_token, marker, 25).await
        .map_err(|e| anyhow::anyhow!("get_updates failed: {}", e))?;
    drop(_permit);

    let updates = response
        .get("updates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let new_marker = response.get("marker").and_then(|v| v.as_i64());

    if !updates.is_empty() {
        tracing::debug!(bot_id = %bot_id, count = updates.len(), "received updates");
    }

    if !updates.is_empty() || new_marker.is_some() {
        // Transactional: insert events + update marker
        let mut tx = pool.begin().await?;
        ingestion::ingest_updates_tx(&mut tx, bot_id, updates, "polling").await
            .map_err(|e| anyhow::anyhow!("Ingestion failed: {}", e))?;

        if let Some(m) = new_marker {
            sqlx::query!("UPDATE bots SET polling_marker = $1 WHERE id = $2", m, bot_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        // Update the cached context's polling_marker to avoid stale reads
        if let Some(m) = new_marker {
            if let Some(ctx) = cached_context {
                ctx.polling_marker = Some(m);
            }
        }
    }

    Ok(PollOutcome::Continue)
}
