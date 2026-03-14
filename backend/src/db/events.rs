use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder, Postgres};
use uuid::Uuid;

use crate::models::Event;

pub struct NewEvent {
    pub bot_id: Uuid,
    pub max_update_id: Option<i64>,
    pub update_type: String,
    pub chat_id: Option<i64>,
    pub chat_type: Option<String>,
    pub sender_id: Option<i64>,
    pub timestamp: i64,
    pub raw_payload: serde_json::Value,
    pub direction: String,
    pub source: String,
}

/// Maximum events per INSERT statement. Each event binds 9 parameters;
/// PostgreSQL supports at most 65535 bind parameters per statement.
/// 1000 * 9 = 9000, well within the limit.
const BATCH_CHUNK_SIZE: usize = 1000;

pub async fn batch_insert(
    pool: &PgPool,
    events: &[NewEvent],
) -> Result<Vec<Uuid>, sqlx::Error> {
    if events.is_empty() {
        return Ok(vec![]);
    }

    let mut all_ids = Vec::with_capacity(events.len());

    for chunk in events.chunks(BATCH_CHUNK_SIZE) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO events (bot_id, max_update_id, update_type, chat_id, sender_id, timestamp, raw_payload, direction, source) "
        );

        qb.push_values(chunk, |mut b, e| {
            b.push_bind(e.bot_id)
             .push_bind(e.max_update_id)
             .push_bind(&e.update_type)
             .push_bind(e.chat_id)
             .push_bind(e.sender_id)
             .push_bind(e.timestamp)
             .push_bind(&e.raw_payload)
             .push_bind(&e.direction)
             .push_bind(&e.source);
        });

        qb.push(" ON CONFLICT DO NOTHING RETURNING id");

        let rows: Vec<(Uuid,)> = qb
            .build_query_as()
            .fetch_all(pool)
            .await?;

        all_ids.extend(rows.into_iter().map(|r| r.0));
    }

    Ok(all_ids)
}

/// Transaction variant of batch_insert. The QueryBuilder logic is intentionally
/// duplicated because sqlx's Executor trait doesn't unify &PgPool and
/// &mut Transaction easily with compile-time checked queries.
pub async fn batch_insert_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    events: &[NewEvent],
) -> Result<Vec<Uuid>, sqlx::Error> {
    if events.is_empty() {
        return Ok(vec![]);
    }

    let mut all_ids = Vec::with_capacity(events.len());

    for chunk in events.chunks(BATCH_CHUNK_SIZE) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO events (bot_id, max_update_id, update_type, chat_id, sender_id, timestamp, raw_payload, direction, source) "
        );

        qb.push_values(chunk, |mut b, e| {
            b.push_bind(e.bot_id)
             .push_bind(e.max_update_id)
             .push_bind(&e.update_type)
             .push_bind(e.chat_id)
             .push_bind(e.sender_id)
             .push_bind(e.timestamp)
             .push_bind(&e.raw_payload)
             .push_bind(&e.direction)
             .push_bind(&e.source);
        });

        qb.push(" ON CONFLICT DO NOTHING RETURNING id");

        let rows: Vec<(Uuid,)> = qb
            .build_query_as()
            .fetch_all(&mut **tx)
            .await?;

        all_ids.extend(rows.into_iter().map(|r| r.0));
    }

    Ok(all_ids)
}

/// Event with explicit created_at — used for history sync where we need
/// events to land in the correct partition and sort position.
pub struct NewHistoryEvent {
    pub bot_id: Uuid,
    pub max_update_id: Option<i64>,
    pub update_type: String,
    pub chat_id: Option<i64>,
    pub sender_id: Option<i64>,
    pub timestamp: i64,
    pub raw_payload: serde_json::Value,
    pub direction: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

/// Batch insert events with explicit created_at values.
/// Dedup uses the existing idx_events_dedup (bot_id, max_update_id, created_at).
/// Since created_at is derived deterministically from the message timestamp,
/// re-syncing the same message produces the same (bot_id, max_update_id, created_at) tuple.
pub async fn batch_insert_history(
    pool: &PgPool,
    events: &[NewHistoryEvent],
) -> Result<Vec<Uuid>, sqlx::Error> {
    if events.is_empty() {
        return Ok(vec![]);
    }

    let mut all_ids = Vec::with_capacity(events.len());

    for chunk in events.chunks(BATCH_CHUNK_SIZE) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO events (bot_id, max_update_id, update_type, chat_id, sender_id, timestamp, raw_payload, direction, source, created_at) "
        );

        qb.push_values(chunk, |mut b, e| {
            b.push_bind(e.bot_id)
             .push_bind(e.max_update_id)
             .push_bind(&e.update_type)
             .push_bind(e.chat_id)
             .push_bind(e.sender_id)
             .push_bind(e.timestamp)
             .push_bind(&e.raw_payload)
             .push_bind(&e.direction)
             .push_bind(&e.source)
             .push_bind(e.created_at);
        });

        qb.push(" ON CONFLICT DO NOTHING RETURNING id");

        let rows: Vec<(Uuid,)> = qb
            .build_query_as()
            .fetch_all(pool)
            .await?;

        all_ids.extend(rows.into_iter().map(|r| r.0));
    }

    Ok(all_ids)
}

pub async fn list_for_bot_filtered(
    pool: &PgPool,
    bot_id: Uuid,
    direction: Option<&str>,
    cursor: Option<(DateTime<Utc>, Uuid)>,
    limit: i64,
) -> Result<Vec<Event>, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT id, bot_id, max_update_id, update_type, chat_id, sender_id, timestamp, raw_payload, created_at, direction, source FROM events WHERE bot_id = "
    );
    builder.push_bind(bot_id);
    if let Some(dir) = direction {
        builder.push(" AND direction = ").push_bind(dir.to_string());
    }
    if let Some((cursor_time, cursor_id)) = cursor {
        builder.push(" AND (created_at, id) < (")
            .push_bind(cursor_time)
            .push(", ")
            .push_bind(cursor_id)
            .push(")");
    }
    builder.push(" ORDER BY created_at DESC, id DESC LIMIT ").push_bind(limit);
    builder.build_query_as::<Event>().fetch_all(pool).await
}

pub async fn list_for_bot_chat_filtered(
    pool: &PgPool,
    bot_id: Uuid,
    chat_id: i64,
    direction: Option<&str>,
    cursor: Option<(DateTime<Utc>, Uuid)>,
    limit: i64,
) -> Result<Vec<Event>, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT id, bot_id, max_update_id, update_type, chat_id, sender_id, timestamp, raw_payload, created_at, direction, source FROM events WHERE bot_id = "
    );
    builder.push_bind(bot_id);
    builder.push(" AND chat_id = ").push_bind(chat_id);
    if let Some(dir) = direction {
        builder.push(" AND direction = ").push_bind(dir.to_string());
    }
    if let Some((cursor_time, cursor_id)) = cursor {
        builder.push(" AND (created_at, id) < (")
            .push_bind(cursor_time)
            .push(", ")
            .push_bind(cursor_id)
            .push(")");
    }
    builder.push(" ORDER BY created_at DESC, id DESC LIMIT ").push_bind(limit);
    builder.build_query_as::<Event>().fetch_all(pool).await
}

pub async fn delete_for_org(tx: &mut sqlx::Transaction<'_, Postgres>, org_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"DELETE FROM events WHERE bot_id IN (SELECT b.id FROM bots b JOIN projects p ON p.id = b.project_id WHERE p.organization_id = $1)"#,
    )
    .bind(org_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_for_project(tx: &mut sqlx::Transaction<'_, Postgres>, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"DELETE FROM events WHERE bot_id IN (SELECT id FROM bots WHERE project_id = $1)"#,
    )
    .bind(project_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Find an event by ID. If `created_at_hint` is provided, it narrows the query
/// to a +/- 1 day window around the hint, enabling partition pruning.
pub async fn find_by_id(
    pool: &PgPool,
    event_id: Uuid,
    bot_id: Uuid,
    created_at_hint: Option<DateTime<Utc>>,
) -> Result<Option<Event>, sqlx::Error> {
    match created_at_hint {
        Some(hint) => {
            sqlx::query_as!(
                Event,
                r#"SELECT id, bot_id, max_update_id, update_type, chat_id, sender_id,
                          timestamp, raw_payload, created_at, direction, source
                   FROM events
                   WHERE id = $1 AND bot_id = $2
                     AND created_at >= $3 AND created_at < $4"#,
                event_id,
                bot_id,
                hint - chrono::Duration::days(1),
                hint + chrono::Duration::days(1),
            )
            .fetch_optional(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Event,
                r#"SELECT id, bot_id, max_update_id, update_type, chat_id, sender_id,
                          timestamp, raw_payload, created_at, direction, source
                   FROM events
                   WHERE id = $1 AND bot_id = $2"#,
                event_id,
                bot_id,
            )
            .fetch_optional(pool)
            .await
        }
    }
}

