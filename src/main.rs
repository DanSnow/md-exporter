mod cache;
mod config;
mod converter;
mod error;
mod routes;

use std::sync::Arc;

use axum::Router;
use cache::{CacheBackend, memory::MemoryCache};
use config::Config;
use minijinja::Environment;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
#[cfg(feature = "redis-cache")]
use tracing::warn;
use utoipa::OpenApi as _;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;
use xxhash_rust::xxh3::xxh3_64;

use routes::openapi::ApiDoc;

pub struct AppState {
    pub config: Arc<Config>,
    pub cache: Arc<dyn CacheBackend>,
    pub typst_env: Arc<Environment<'static>>,
    pub typst_hash: u64,
    pub pandoc_version: String,
    pub typst_version: String,
    cache_backend: &'static str,
}

impl AppState {
    pub fn cache_backend_name(&self) -> &'static str {
        self.cache_backend
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    let pandoc_bin = config.pandoc_bin.as_deref().unwrap_or("pandoc");
    let (pandoc_version, typst_version) = tokio::try_join!(
        routes::health::probe_version(pandoc_bin),
        routes::health::probe_version(&config.typst_bin),
    )
    .map_err(|e| anyhow::anyhow!("startup binary check failed: {}", e))?;

    let template_bytes = std::fs::read(&config.typst_template).map_err(|e| {
        anyhow::anyhow!(
            "failed to read typst template '{}': {}",
            config.typst_template,
            e
        )
    })?;
    let typst_hash = xxh3_64(&template_bytes);

    let template_src = String::from_utf8(template_bytes)
        .map_err(|e| anyhow::anyhow!("typst template is not valid UTF-8: {}", e))?;
    let mut env = Environment::new();
    env.add_template_owned("default.typ", template_src)?;
    let typst_env = Arc::new(env);

    let memory = Arc::new(MemoryCache::new(
        config.cache_max_entries,
        config.cache_ttl_secs,
    ));
    let (cache, cache_backend): (Arc<dyn CacheBackend>, &'static str) = {
        #[cfg(feature = "redis-cache")]
        if let Some(ref redis_url) = config.redis_url {
            use cache::redis::{RedisCache, TwoLayerCache};
            match RedisCache::new(redis_url, config.cache_ttl_secs).await {
                Ok(redis) => (Arc::new(TwoLayerCache::new(redis, memory)), "redis"),
                Err(e) => {
                    warn!(
                        "Redis connection failed ({}), falling back to memory cache",
                        e
                    );
                    (memory, "memory")
                }
            }
        } else {
            #[cfg(feature = "redis-cache")]
            warn!(
                "redis-cache feature is compiled in but REDIS_URL is not set; using memory cache"
            );
            (memory, "memory")
        }

        #[cfg(not(feature = "redis-cache"))]
        {
            (memory, "memory")
        }
    };

    let port = config.port;
    let state = Arc::new(AppState {
        config: Arc::new(config),
        cache,
        typst_env,
        typst_hash,
        pandoc_version,
        typst_version,
        cache_backend,
    });

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(routes::export::export_handler))
        .routes(routes!(routes::health::health_handler))
        .with_state(state)
        .split_for_parts();

    let app: Router = router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
        );

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tracing::info!("Listening on port {}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
