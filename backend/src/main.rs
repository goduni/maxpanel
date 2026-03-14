use std::sync::Arc;
use std::time::Duration;

use axum::http::{header, Method};
use axum::middleware as axum_middleware;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use max_dashboard_backend::app_state::AppState;
use max_dashboard_backend::config::Config;
use max_dashboard_backend::middleware::security_headers;
use max_dashboard_backend::router::build_router;
use max_dashboard_backend::workers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let config = Config::from_env()?;
    tracing::info!(listen_addr = %config.listen_addr, env = ?config.app_env, "Starting server");

    // Database pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(Duration::from_secs(config.database_acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database_url)
        .await?;

    tracing::info!("Database connected");

    // Run migrations
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("Migrations applied");

    let cancel = CancellationToken::new();
    let state = AppState::new(pool.clone(), config.clone(), cancel.clone());

    // CORS
    let cors_origins: Vec<_> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(cors_origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    // Build router
    let app = build_router(state.clone())
        .layer(axum_middleware::from_fn(security_headers::security_headers))
        .layer(cors)
        .layer(TraceLayer::new_for_http()
            .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO)))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024)); // 2 MiB

    // Spawn background workers
    let worker_cancel = cancel.clone();
    let worker_pool = pool.clone();
    let worker_config = Arc::new(config.clone());
    let worker_http = state.http_client.clone();

    tokio::spawn(async move {
        workers::partition_manager::run_partition_manager(worker_pool.clone(), worker_cancel.clone()).await;
    });

    let token_cleanup_cancel = cancel.clone();
    let token_cleanup_pool = pool.clone();
    tokio::spawn(async move {
        workers::token_cleanup::run_token_cleanup(token_cleanup_pool, token_cleanup_cancel).await;
    });

    let polling_cancel = cancel.clone();
    let polling_pool = pool.clone();
    let polling_config = Arc::clone(&worker_config);
    let polling_http = worker_http.clone();

    tokio::spawn(async move {
        workers::polling::run_polling_supervisor(polling_pool, polling_config, polling_http, polling_cancel).await;
    });

    {
        let cache = Arc::clone(&state.video_url_cache);
        let cancel = cancel.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = interval.tick() => {
                        cache.retain(|_, (_, ts)| ts.elapsed() < Duration::from_secs(300));
                    }
                }
            }
        });
    }

    // Start HTTP server
    let listener = TcpListener::bind(&config.listen_addr).await?;
    tracing::info!("Listening on {}", config.listen_addr);

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .with_graceful_shutdown(shutdown_signal(cancel.clone()))
        .await?;

    // Shutdown
    tracing::info!("Shutting down...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    pool.close().await;

    Ok(())
}

async fn shutdown_signal(cancel: CancellationToken) {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl+c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
    cancel.cancel();
}
