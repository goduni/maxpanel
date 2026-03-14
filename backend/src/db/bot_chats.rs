use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder, Postgres};
use uuid::Uuid;

use crate::models::BotChat;

pub struct UpsertChat {
    pub bot_id: Uuid,
    pub chat_id: i64,
    pub chat_type: Option<String>,
    pub title: Option<String>,
    pub icon_url: Option<String>,
    pub participants: Option<i32>,
    pub last_event_at: Option<DateTime<Utc>>,
}

pub struct EventChat {
    pub bot_id: Uuid,
    pub chat_id: i64,
    pub chat_type: Option<String>,
    pub title: Option<String>,
}

/// Batch upsert chats discovered from events. Single INSERT replaces N+1 individual upserts.
pub async fn batch_upsert_from_events(
    pool: &PgPool,
    chats: &[EventChat],
) -> Result<(), sqlx::Error> {
    if chats.is_empty() {
        return Ok(());
    }
    let now = Utc::now();
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO bot_chats (bot_id, chat_id, chat_type, title, last_event_at) "
    );
    qb.push_values(chats, |mut b, c| {
        b.push_bind(c.bot_id)
         .push_bind(c.chat_id)
         .push_bind(&c.chat_type)
         .push_bind(&c.title)
         .push_bind(now);
    });
    qb.push(
        " ON CONFLICT (bot_id, chat_id) DO UPDATE SET \
         last_event_at = GREATEST(bot_chats.last_event_at, EXCLUDED.last_event_at), \
         chat_type = COALESCE(EXCLUDED.chat_type, bot_chats.chat_type), \
         title = COALESCE(EXCLUDED.title, bot_chats.title)"
    );
    qb.build().execute(pool).await?;
    Ok(())
}

/// Transaction variant of batch_upsert_from_events.
pub async fn batch_upsert_from_events_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    chats: &[EventChat],
) -> Result<(), sqlx::Error> {
    if chats.is_empty() {
        return Ok(());
    }
    let now = Utc::now();
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO bot_chats (bot_id, chat_id, chat_type, title, last_event_at) "
    );
    qb.push_values(chats, |mut b, c| {
        b.push_bind(c.bot_id)
         .push_bind(c.chat_id)
         .push_bind(&c.chat_type)
         .push_bind(&c.title)
         .push_bind(now);
    });
    qb.push(
        " ON CONFLICT (bot_id, chat_id) DO UPDATE SET \
         last_event_at = GREATEST(bot_chats.last_event_at, EXCLUDED.last_event_at), \
         chat_type = COALESCE(EXCLUDED.chat_type, bot_chats.chat_type), \
         title = COALESCE(EXCLUDED.title, bot_chats.title)"
    );
    qb.build().execute(&mut **tx).await?;
    Ok(())
}

/// Batch upsert chats from Max API sync. Updates metadata but preserves last_event_at.
pub async fn batch_upsert_from_sync(
    pool: &PgPool,
    chats: &[UpsertChat],
) -> Result<(), sqlx::Error> {
    if chats.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    for chunk in chats.chunks(500) {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO bot_chats (bot_id, chat_id, chat_type, title, icon_url, participants, synced_at) "
        );

        qb.push_values(chunk, |mut b, c| {
            b.push_bind(c.bot_id)
             .push_bind(c.chat_id)
             .push_bind(&c.chat_type)
             .push_bind(&c.title)
             .push_bind(&c.icon_url)
             .push_bind(c.participants)
             .push_bind(now);
        });

        qb.push(
            " ON CONFLICT (bot_id, chat_id) DO UPDATE SET \
             chat_type = COALESCE(EXCLUDED.chat_type, bot_chats.chat_type), \
             title = COALESCE(EXCLUDED.title, bot_chats.title), \
             icon_url = COALESCE(EXCLUDED.icon_url, bot_chats.icon_url), \
             participants = COALESCE(EXCLUDED.participants, bot_chats.participants), \
             synced_at = EXCLUDED.synced_at"
        );

        qb.build().execute(pool).await?;
    }

    Ok(())
}

pub async fn list_for_bot(
    pool: &PgPool,
    bot_id: Uuid,
    cursor: Option<(DateTime<Utc>, i64)>,
    limit: i64,
    search: Option<&str>,
) -> Result<Vec<BotChat>, sqlx::Error> {
    let mut builder = QueryBuilder::new(
        "SELECT bot_id, chat_id, chat_type, title, icon_url, participants, last_event_at, synced_at FROM bot_chats WHERE bot_id = "
    );
    builder.push_bind(bot_id);
    if let Some(q) = search {
        builder.push(" AND (title ILIKE ");
        builder.push_bind(format!("%{}%", q.replace('%', "\\%").replace('_', "\\_")));
        builder.push(" OR CAST(chat_id AS TEXT) LIKE ");
        builder.push_bind(format!("%{}%", q.replace('%', "\\%").replace('_', "\\_")));
        builder.push(")");
    }
    if let Some((cursor_time, cursor_chat_id)) = cursor {
        builder.push(" AND (COALESCE(last_event_at, synced_at), chat_id) < (")
            .push_bind(cursor_time)
            .push(", ")
            .push_bind(cursor_chat_id)
            .push(")");
    }
    builder.push(" ORDER BY COALESCE(last_event_at, synced_at) DESC, chat_id DESC LIMIT ").push_bind(limit);
    builder.build_query_as::<BotChat>().fetch_all(pool).await
}
