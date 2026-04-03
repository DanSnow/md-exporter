# PRD: Markdown Export Service (md-export)

## Overview

A lightweight HTTP service that accepts a Markdown string, converts it to PDF or DOCX via Pandoc + Typst CLI, and returns the file as a binary response. Provides in-memory caching by default, with optional Redis cache support via a Cargo feature flag.

---

## Tech Stack

| Component         | Choice                             |
| ----------------- | ---------------------------------- |
| Language          | Rust                               |
| HTTP framework    | Axum                               |
| Async runtime     | Tokio                              |
| CLI execution     | `tokio::process::Command`          |
| In-memory cache   | `moka` (async TTL cache)           |
| Redis client      | `fred` (optional, feature-gated)   |
| Cache key hashing | `xxhash` (xxh3)                    |
| Config            | `figment2` + environment variables |
| Error handling    | `anyhow` + `thiserror`             |
| Logging           | `tracing` + `tracing-subscriber`   |
| Temp files        | `tempfile`                         |
| PDF templating    | `minijinja` (runtime Jinja engine) |
| OpenAPI spec      | `utoipa` + `utoipa-axum`           |

---

## API

### POST `/export`

**Request body**

```json
{
  "markdown": "# Hello\n\nThis is **markdown**.",
  "format": "pdf",
  "filename": "report.pdf",
  "inline": false
}
```

| Field      | Type                | Required | Default                          | Description                                                                                           |
| ---------- | ------------------- | -------- | -------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `markdown` | string              | ✅       | —                                | Markdown content to convert                                                                           |
| `format`   | `"pdf"` \| `"docx"` | ✅       | —                                | Output format                                                                                         |
| `filename` | string              | ❌       | `"export.pdf"` / `"export.docx"` | Filename used in the `Content-Disposition` header                                                     |
| `inline`   | boolean             | ❌       | `false`                          | When `true`, sets `Content-Disposition: inline` so browsers render the file instead of downloading it |

**Success response**

- Status: `200 OK`
- `Content-Type`: `application/pdf` or `application/vnd.openxmlformats-officedocument.wordprocessingml.document`
- `Content-Disposition`: `attachment; filename="<filename>"` (default) or `inline` when `inline: true`
- Body: raw file binary

**Error response**

```json
{
  "error": "conversion_failed",
  "message": "pandoc exited with code 1: ..."
}
```

| HTTP Status | Error Code          | Description                    |
| ----------- | ------------------- | ------------------------------ |
| `400`       | `invalid_request`   | Invalid format, empty markdown |
| `422`       | `conversion_failed` | Pandoc / Typst process failed  |
| `500`       | `internal_error`    | Unexpected server error        |

---

### GET `/health`

Returns service status and confirms that Pandoc and Typst binaries are executable.

```json
{
  "status": "ok",
  "pandoc_version": "3.x.x",
  "typst_version": "0.x.x",
  "cache_backend": "memory"
}
```

---

### GET `/openapi.json`

Returns the OpenAPI 3.x specification as JSON.

### GET `/swagger`

Serves the Swagger UI for interactive API exploration.

---

## OpenAPI Spec Generation

OpenAPI documentation is generated at compile time using `utoipa` proc macros and wired into the router via `utoipa-axum`.

### Dependencies

```toml
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-axum = "0.1"
utoipa-swagger-ui = { version = "8", features = ["axum"] }
```

### Approach

- Request/response structs derive `utoipa::ToSchema`
- Each handler is annotated with `#[utoipa::path(...)]`
- Routes are registered via `OpenApiRouter` (from `utoipa-axum`), which collects path metadata automatically
- A top-level `#[derive(OpenApi)]` struct declares tags and assembles the final spec
- `split_for_parts()` separates the Axum router from the `OpenApi` object; the spec is then served at `/openapi.json` and exposed through Swagger UI at `/swagger`

### Example skeleton

```rust
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(tags((name = "export", description = "Markdown export API")))]
struct ApiDoc;

let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
    .routes(routes!(export_handler))
    .routes(routes!(health_handler))
    .split_for_parts();

let app = router.merge(
    SwaggerUi::new("/swagger").url("/openapi.json", api),
);
```

---

## Cache Design

### Cache Key

PDF:  `xxh3( markdown_content + "\0" + "pdf"  + "\0" + typst_hash )`
DOCX: `xxh3( markdown_content + "\0" + "docx" )`

`typst_hash` is computed once at startup by hashing the raw Typst template file contents using xxh3. It is stored in the application state and automatically reflects any template change without manual version bumping. DOCX conversion has no template, so the hash is omitted from DOCX cache keys.

Startup hash computation:

```rust
// Computed once, stored in AppState
let typst_template_src = std::fs::read(&config.typst_template)?;
let typst_hash = xxh3(&typst_template_src);
```

### In-memory Cache (default)

- Uses `moka::future::Cache`
- TTL: 1 hour (configurable)
- Max entries: 500 (configurable)
- Value: `Bytes` (PDF or DOCX binary)

### Redis Cache (optional)

Enabled via Cargo feature flag `redis-cache`:

```toml
cargo build --features redis-cache
```

- Key format: `md-export:{cache_key_hex}`
- Value: raw binary via `SET key bytes EX ttl`
- TTL: same as in-memory setting

Cache lookup order when Redis is enabled:

```
Request → Redis → In-memory → Convert → Write to both layers
```

### Cache Headers

Responses include debug headers:

```
X-Cache: HIT | MISS
X-Cache-Backend: memory | redis
```

---

## Conversion Logic

### PDF

The Typst template (`default.typ`) is a minijinja template. Before invoking Pandoc, the service renders it with any per-request variables (e.g. metadata passed in the request), writes the rendered source to a temp file, and passes that to Pandoc:

```rust
let rendered = env.get_template("default.typ")?.render(context)?;
// write rendered to a NamedTempFile, pass path to pandoc
```

```bash
pandoc --from=markdown --to=pdf \
  --pdf-engine=typst \
  --template={rendered_typst_tmp} \
  -o {output_tmp} \
  {input_tmp}
```

### DOCX

```bash
pandoc --from=markdown --to=docx \
  --reference-doc={REFERENCE_DOCX} \
  -o {output_tmp} \
  {input_tmp}
```

### Implementation Notes

- Input and output use `tempfile::NamedTempFile`; files are automatically deleted when the handle is dropped
- Pandoc process timeout: 30 seconds (configurable)
- stderr is fully captured and included in error responses on failure

---

## Configuration (Environment Variables)

| Variable                  | Default                    | Description                                                                                               |
| ------------------------- | -------------------------- | --------------------------------------------------------------------------------------------------------- |
| `PORT`                    | `8080`                     | HTTP listen port                                                                                          |
| `PANDOC_BIN`              | _(unset)_                  | Path to Pandoc binary; when unset, `pandoc` is looked up from `$PATH`                                    |
| `TYPST_BIN`               | `typst`                    | Path to Typst binary                                                                                      |
| `TYPST_TEMPLATE`          | `templates/default.typ`    | Typst PDF template path                                                                                   |
| `REFERENCE_DOCX`          | `templates/reference.docx` | DOCX style reference path                                                                                 |
| `CACHE_TTL_SECS`          | `3600`                     | Cache TTL in seconds                                                                                      |
| `CACHE_MAX_ENTRIES`       | `500`                      | Max in-memory cache entries                                                                               |
| `CONVERSION_TIMEOUT_SECS` | `30`                       | Pandoc execution timeout in seconds                                                                       |
| `REDIS_URL`               | _(unset)_                  | Redis URL; when set at runtime **and** the `redis-cache` feature is compiled in, Redis cache is activated |
| `LOG_LEVEL`               | `info`                     | Tracing log level                                                                                         |

---

## Project Structure

```
md-export/
├── Cargo.toml
├── templates/
│   ├── default.typ          # Typst PDF template
│   └── reference.docx       # DOCX style reference
└── src/
    ├── main.rs
    ├── config.rs            # Environment variable loading
    ├── error.rs             # AppError + IntoResponse
    ├── converter.rs         # Pandoc spawn logic
    ├── cache/
    │   ├── mod.rs           # CacheBackend trait
    │   ├── memory.rs        # moka implementation
    │   └── redis.rs         # fred implementation (feature-gated)
    └── routes/
        ├── mod.rs
        ├── export.rs        # POST /export
        ├── health.rs        # GET /health
        └── openapi.rs       # GET /openapi.json + /swagger
```

---

## Dockerfile

```dockerfile
FROM rust:1.78 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y pandoc wget && \
    wget https://github.com/typst/typst/releases/latest/download/typst-x86_64-unknown-linux-musl.tar.xz && \
    tar xf typst-*.tar.xz && mv typst-*/typst /usr/local/bin/ && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/md-export /usr/local/bin/
COPY templates/ /app/templates/
WORKDIR /app
EXPOSE 8080
CMD ["md-export"]
```

---

## Kubernetes Resource Recommendation

```yaml
resources:
  requests:
    cpu: '250m'
    memory: '256Mi'
  limits:
    cpu: '1000m'
    memory: '1Gi'
```

---

## Out of Scope

- Authentication / API key validation (handled by upstream API gateway)
- Async job queue (synchronous response is sufficient for now)
- Object storage offload for large cached files (revisit if Redis values exceed ~1MB consistently)
- DOCX → PDF conversion (input is always Markdown)
