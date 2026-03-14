use chrono::{Datelike, Utc};
use sqlx::PgPool;
use std::sync::OnceLock;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Creates next month's event partition if it doesn't exist.
/// Runs once at startup, then daily.
pub async fn run_partition_manager(pool: PgPool, cancel: CancellationToken) {
    // Run immediately
    if let Err(e) = ensure_next_partition(&pool).await {
        tracing::error!(error = %e, "Failed to create partition at startup");
    }

    let mut interval = tokio::time::interval(Duration::from_secs(86400));
    interval.tick().await; // skip the immediate tick (already ran above)

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Partition manager shutting down");
                return;
            }
            _ = interval.tick() => {
                if let Err(e) = ensure_next_partition(&pool).await {
                    tracing::error!(error = %e, "Failed to create partition");
                }
            }
        }
    }
}

/// Validate a partition name matches expected format: events_YYYY_MM
fn validate_partition_name(name: &str) -> bool {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| regex_lite::Regex::new(r"^events_\d{4}_\d{2}$").unwrap()).is_match(name)
}

/// Validate a date string matches YYYY-MM-DD format
fn validate_date(date: &str) -> bool {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| regex_lite::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap()).is_match(date)
}

async fn create_partition_if_missing(
    pool: &PgPool,
    partition_name: &str,
    from_date: &str,
    to_date: &str,
) -> Result<(), sqlx::Error> {
    if !validate_partition_name(partition_name) {
        return Err(sqlx::Error::Protocol(format!("Invalid partition name: {}", partition_name)));
    }
    if !validate_date(from_date) {
        return Err(sqlx::Error::Protocol(format!("Invalid from_date: {}", from_date)));
    }
    if !validate_date(to_date) {
        return Err(sqlx::Error::Protocol(format!("Invalid to_date: {}", to_date)));
    }

    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_class WHERE relname = $1)"
    )
    .bind(partition_name)
    .fetch_one(pool)
    .await?;

    if !exists {
        // Partition names and dates are validated above to be safe identifiers.
        // PostgreSQL DDL does not support parameterized identifiers, so format! is required.
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" PARTITION OF events FOR VALUES FROM ('{}') TO ('{}')",
            partition_name, from_date, to_date
        );
        sqlx::query(&sql).execute(pool).await?;
        tracing::info!(partition = %partition_name, "Created event partition");
    }

    Ok(())
}

fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

async fn ensure_next_partition(pool: &PgPool) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    let (m1_year, m1_month) = (now.year(), now.month());
    let (m2_year, m2_month) = next_month(m1_year, m1_month);
    let (m3_year, m3_month) = next_month(m2_year, m2_month);
    let (m4_year, m4_month) = next_month(m3_year, m3_month);

    // Current month partition
    let current_name = format!("events_{}_{:02}", m1_year, m1_month);
    let current_from = format!("{}-{:02}-01", m1_year, m1_month);
    let current_to = format!("{}-{:02}-01", m2_year, m2_month);
    create_partition_if_missing(pool, &current_name, &current_from, &current_to).await?;

    // Next month partition
    let next_name = format!("events_{}_{:02}", m2_year, m2_month);
    let next_from = format!("{}-{:02}-01", m2_year, m2_month);
    let next_to = format!("{}-{:02}-01", m3_year, m3_month);
    create_partition_if_missing(pool, &next_name, &next_from, &next_to).await?;

    // Month after next partition
    let after_name = format!("events_{}_{:02}", m3_year, m3_month);
    let after_from = format!("{}-{:02}-01", m3_year, m3_month);
    let after_to = format!("{}-{:02}-01", m4_year, m4_month);
    create_partition_if_missing(pool, &after_name, &after_from, &after_to).await?;

    Ok(())
}
