## 1. Project Scaffolding

- [x] 1.1 Create `Cargo.toml` with binary target `md-export`; module structure matches PRD verbatim (`src/config.rs`, `src/error.rs`, `src/converter.rs`, `src/cache/`, `src/routes/`); add all required dependencies (`axum`, `tokio`, `moka`, `xxhash-rust`, `figment`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, `tempfile`, `minijinja`, `bytes`, `utoipa`, `utoipa-axum`, `utoipa-swagger-ui`, `async-trait`), and optional `fred` dependency behind the `redis-cache` feature flag
- [x] 1.2 Create stub `src/main.rs` that compiles cleanly (empty `main` function)
- [x] 1.3 Create `templates/default.typ` with a minimal Typst template (title, body variable, basic page setup); this file is loaded for typst template rendered via minijinja before each PDF conversion
- [x] 1.4 Create `templates/reference.docx` placeholder (can be a minimal valid DOCX; Pandoc will use it as a style reference)

## 2. Configuration

- [x] 2.1 Create `src/config.rs`: define `Config` struct with fields for `port` (u16, default 8080), `pandoc_bin` (Option<String>), `typst_bin` (String, default "typst"), `typst_template` (String, default "templates/default.typ"), `reference_docx` (String, default "templates/reference.docx"), `cache_ttl_secs` (u64, default 3600), `cache_max_entries` (u64, default 500), `conversion_timeout_secs` (u64, default 30), `redis_url` (Option<String>, feature-gated), `log_level` (String, default "info")
- [x] 2.2 Implement `Config::from_env()`: configuration via figment2 + environment variables only using `Env::raw()` provider; return `anyhow::Result<Config>`

## 3. Error Handling

- [x] 3.1 Create `src/error.rs`: error handling: AppError enum + IntoResponse — variants `InvalidRequest(String)`, `ConversionFailed(String)`, `InternalError(anyhow::Error)`
- [x] 3.2 Implement `IntoResponse` for `AppError` mapping to HTTP 400/422/500 with JSON body `{"error": "<code>", "message": "<msg>"}`

## 4. Cache Abstraction

- [x] 4.1 Create `src/cache/mod.rs`: CacheBackend is a trait, not an enum — define async trait with `get(&self, key: u64) -> Option<Bytes>` and `set(&self, key: u64, value: Bytes)` methods; define `CacheResult` enum (`Hit`, `Miss`) for tracking hit/miss state
- [x] 4.2 Create `src/cache/memory.rs`: implement `MemoryCache` using `moka::future::Cache<u64, Bytes>` with TTL and max-entries from config; implement `CacheBackend` for it — satisfies "cache conversion results in memory" requirement
- [x] 4.3 Create `src/cache/redis.rs` (feature-gated `redis-cache`): implement `RedisCache` using `fred` client; implement `CacheBackend`; implement `TwoLayerCache` — two-layer Redis cache is a single struct, not trait composition — wraps `RedisCache` + `MemoryCache` with lookup order Redis → memory → miss; write to both on miss — satisfies "optional Redis cache layer via feature flag" requirement
- [x] 4.4 Implement cache key: u64 from xxh3 — function in `src/cache/mod.rs`: `fn compute_key(markdown: &str, format: &str, typst_hash: Option<u64>) -> u64` using `xxhash_rust::xxh3::xxh3_64` — satisfies "compute cache key via xxh3 hash" requirement

## 5. Converter

- [x] 5.1 Create `src/converter.rs`: define `ConvertRequest` struct with `markdown: String`, `format: ExportFormat`; define `ExportFormat` enum (`Pdf`, `Docx`)
- [x] 5.2 Implement `async fn convert(req: ConvertRequest, config: &Config, typst_env: &minijinja::Environment) -> Result<Bytes, AppError>` that: (a) for PDF — typst template rendered via minijinja before each PDF conversion: render template to a `NamedTempFile`, write Markdown to second `NamedTempFile`; temp files: hold `NamedTempFile` as a local across `.await` so files stay alive until Pandoc completes; spawn Pandoc with `--pdf-engine=typst --template=<rendered_tmp>`; (b) for DOCX — write Markdown to a `NamedTempFile`, spawn Pandoc with `--reference-doc=<reference_docx>` — satisfies "convert Markdown to PDF via Pandoc + Typst" and "convert Markdown to DOCX via Pandoc" requirements
- [x] 5.3 Implement Pandoc timeout: wrap `Command::output()` with `tokio::time::timeout(Duration::from_secs(config.conversion_timeout_secs), ...)`, kill process and return `ConversionFailed("timeout")` on expiry — satisfies "Pandoc timeout" scenario
- [x] 5.4 Capture Pandoc stderr and include full stderr text in `ConversionFailed` error message on non-zero exit

## 6. Application State

- [x] 6.1 Create `AppState` struct in `src/main.rs` (or `src/state.rs`) holding `config: Arc<Config>`, `cache: Arc<dyn CacheBackend>`, `typst_env: Arc<minijinja::Environment<'static>>`, `typst_hash: u64`
- [x] 6.2 Implement startup logic in `main`: load config, compute `typst_hash` by reading template bytes and hashing with xxh3 — satisfies "Typst template hash computed at startup" requirement; load minijinja environment with `default.typ` template; construct cache backend (memory, or two-layer if `redis-cache` feature + `REDIS_URL` present); log warning if `REDIS_URL` absent with `redis-cache` compiled — satisfies "REDIS_URL absent with feature compiled" scenario

## 7. Export Route

- [x] 7.1 Create `src/routes/export.rs`: define `ExportRequest` struct deriving `Deserialize` + `ToSchema` with fields `markdown: String`, `format: String`, `filename: Option<String>`, `inline: Option<bool>`
- [x] 7.2 Implement `async fn export_handler(State(state): State<Arc<AppState>>, Json(req): Json<ExportRequest>) -> impl IntoResponse` that: validates non-empty markdown and valid format — satisfies "accept Markdown export request" and "missing/empty/invalid field" scenarios
- [x] 7.3 In `export_handler`: compute cache key, check cache, on hit return cached bytes with `X-Cache: HIT` header — satisfies "cache hit skips conversion" scenario; on miss run converter, store result, return with `X-Cache: MISS` — satisfies "cache miss triggers conversion" scenario
- [x] 7.4 In `export_handler`: set `Content-Type` header (`application/pdf` or `application/vnd.openxmlformats-officedocument.wordprocessingml.document`) and `Content-Disposition` header based on `filename` and `inline` fields — satisfies "return file with correct Content-Disposition" requirement
- [x] 7.5 In `export_handler`: set `X-Cache-Backend` header to `"memory"` or `"redis"` based on which layer produced a hit — satisfies "expose cache debug headers" requirement
- [x] 7.6 Annotate `export_handler` with `#[utoipa::path(post, path = "/export", ...)]` — satisfies "export endpoint in spec" scenario

## 8. Health Route

- [x] 8.1 Create `src/routes/health.rs`: define `HealthResponse` struct deriving `Serialize` + `ToSchema` with fields `status: String`, `pandoc_version: String`, `typst_version: String`, `cache_backend: String`
- [x] 8.2 Implement `async fn health_handler` that probes Pandoc and Typst by running `pandoc --version` and `typst --version`, extracts version strings, returns `200 OK` with `HealthResponse` — satisfies "health endpoint reports service status" requirement
- [x] 8.3 In `health_handler`: return `500` with error message if either binary is missing or not executable — satisfies "Pandoc missing" and "Typst missing" scenarios
- [x] 8.4 In `health_handler`: populate `cache_backend` field from `AppState` — satisfies "health endpoint reports cache backend" requirement
- [x] 8.5 Annotate `health_handler` with `#[utoipa::path(get, path = "/health", ...)]` — satisfies "health endpoint in spec" scenario

## 9. OpenAPI Route

- [x] 9.1 Create `src/routes/openapi.rs`: define `ApiDoc` struct deriving `OpenApi` with tags for the export API; OpenAPI via utoipa + utoipa-axum
- [x] 9.2 Wire routes using `OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(export_handler)).routes(routes!(health_handler))`; call `split_for_parts()` to separate Axum router from `OpenApi` object — satisfies "all public endpoints documented in OpenAPI spec" requirement
- [x] 9.3 Merge `SwaggerUi::new("/swagger").url("/openapi.json", api)` into the router — satisfies "serve OpenAPI 3.x spec at /openapi.json" and "serve Swagger UI at /swagger" requirements

## 10. Server Startup

- [x] 10.1 In `main`: initialize tracing subscriber with log level from config; bind `TcpListener` on configured port; call `axum::serve(listener, app).await`
- [x] 10.2 Verify the service compiles and starts cleanly with `cargo run`; confirm `GET /health` returns `200 OK` and `GET /openapi.json` returns a valid JSON spec

## 11. Redis Feature

- [x] 11.1 Implement `src/cache/redis.rs` fully: connect `fred` client to `REDIS_URL` on startup, implement `RedisCache::get` (GET key) and `RedisCache::set` (SET key value EX ttl) with key format `md-export:<key_hex>`
- [x] 11.2 Implement `TwoLayerCache` in `src/cache/redis.rs`: on `get`, check Redis first, then memory; on `set`, write to both — satisfies "Redis hit", "Redis miss memory miss", and "Redis miss memory hit" scenarios
- [x] 11.3 Wire `TwoLayerCache` into startup when `redis-cache` feature is active and `REDIS_URL` is set; test `cargo build --features redis-cache` compiles cleanly
