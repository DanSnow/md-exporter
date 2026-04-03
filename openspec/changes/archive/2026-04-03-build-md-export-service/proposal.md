## Why

A lightweight HTTP service is needed to convert Markdown to PDF and DOCX on demand. No such service currently exists in the project; this builds it from scratch using Rust, Axum, Pandoc, and Typst.

## What Changes

- Introduces a new Rust binary `md-export` served via Axum on configurable port
- `POST /export` accepts Markdown + format, returns a binary PDF or DOCX file
- `GET /health` confirms binary availability (Pandoc, Typst) and cache backend
- `GET /openapi.json` and `GET /swagger` serve the OpenAPI spec and Swagger UI
- In-memory TTL cache (moka) keyed by xxh3 hash of content + format + template hash
- Optional Redis cache layer via `redis-cache` Cargo feature flag
- Pandoc invoked via `tokio::process::Command`; PDF uses Typst as the PDF engine with a minijinja-rendered template; DOCX uses a reference `.docx` file
- All configuration via environment variables; binaries looked up from `$PATH` by default

## Non-Goals

- Authentication / API key validation (handled upstream by API gateway)
- Async job queue (synchronous response is sufficient)
- Object storage offload for large cached files
- DOCX → PDF conversion (input is always Markdown)

## Capabilities

### New Capabilities

- `markdown-export`: Convert Markdown to PDF or DOCX via Pandoc + Typst and return the binary file over HTTP
- `export-cache`: Cache conversion results in-memory (moka) with optional Redis layer, keyed by content hash
- `health-check`: Report service liveness and confirm Pandoc/Typst binary availability
- `openapi-docs`: Serve OpenAPI 3.x spec at `/openapi.json` and Swagger UI at `/swagger`

### Modified Capabilities

(none)

## Impact

- Affected specs: `markdown-export`, `export-cache`, `health-check`, `openapi-docs`
- Affected code: `src/main.rs`, `src/config.rs`, `src/error.rs`, `src/converter.rs`, `src/cache/mod.rs`, `src/cache/memory.rs`, `src/cache/redis.rs`, `src/routes/export.rs`, `src/routes/health.rs`, `src/routes/openapi.rs`, `Cargo.toml`, `templates/default.typ`, `templates/reference.docx`
- New dependencies: `axum`, `tokio`, `moka`, `fred` (feature-gated), `xxhash-rust`, `figment22`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, `tempfile`, `minijinja`, `utoipa`, `utoipa-axum`, `utoipa-swagger-ui`, `bytes`
