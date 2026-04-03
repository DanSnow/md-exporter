## Context

Greenfield Rust service. No existing source code. The PRD defines all requirements; this document records key technical decisions made before coding begins so implementers don't have to re-derive them.

Runtime: Tokio async. Framework: Axum. All I/O (Pandoc spawn, cache, file writes) is async-safe.

## Goals / Non-Goals

**Goals:**

- Define module boundaries and data flow for the service
- Record key technical decisions with rationale
- Establish the cache abstraction contract
- Clarify temp-file lifetime management across async boundaries

**Non-Goals:**

- Line-by-line implementation guidance (that's tasks)
- Authentication (out of scope per PRD)
- Redis feature flag implementation details (wired in last, after core is working)

## Decisions

### Module structure matches PRD verbatim

`src/` layout: `main.rs`, `config.rs`, `error.rs`, `converter.rs`, `cache/` (mod + memory + redis), `routes/` (mod + export + health + openapi).

**Rationale**: The PRD already encodes the right dependency order (config/error have no internal deps; converter depends on config; cache depends on config; routes depend on all three). Following it avoids discussion.

**Alternative considered**: Flat module structure — rejected because cache has two implementations that need separate files.

### CacheBackend is a trait, not an enum

```rust
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get(&self, key: u64) -> Option<Bytes>;
    async fn set(&self, key: u64, value: Bytes);
}
```

`AppState` holds `Arc<dyn CacheBackend>`. With `redis-cache` feature enabled, the backend is a two-layer struct (Redis → memory) that implements the same trait.

**Rationale**: Keeps route handlers ignorant of which backend is active. Adding Redis becomes additive — no changes to handler code.

**Alternative considered**: Enum with `Memory` and `Redis` variants — rejected because match arms in handler code would need updating whenever a new backend is added.

### Two-layer Redis cache is a single struct, not trait composition

When `redis-cache` is enabled, a `TwoLayerCache` struct wraps both a `MemoryCache` and a `RedisCache` and implements `CacheBackend`. Lookup order: Redis → memory → convert → write both.

**Rationale**: The lookup order is fixed by the PRD. There's no need for composable layering.

### Cache key: u64 from xxh3

PDF key: `xxh3(markdown + "\0" + "pdf" + "\0" + typst_hash_hex)`
DOCX key: `xxh3(markdown + "\0" + "docx")`

`typst_hash` is computed once at startup by reading `templates/default.typ` and hashing its bytes. Stored in `AppState` alongside the minijinja `Environment`.

**Rationale**: Template changes invalidate PDF cache automatically without manual version bumping. DOCX has no template, so no hash needed.

### Temp files: hold `NamedTempFile` as a local across `.await`

Pandoc needs named paths on disk. We write input Markdown to a `NamedTempFile`, pass `.path()` to the `Command`, and hold the handle as a local variable in the same async block. Rust's future captures locals held across `.await`, so the file stays alive until `tokio::process::Command::output().await` completes.

**No `.keep()` is needed.** After the `await`, the handle is dropped and the OS cleans up.

For PDF, the rendered Typst template is also written to a `NamedTempFile` before invoking Pandoc.

**Alternative considered**: Writing to a fixed `/tmp/md-export-*` path — rejected because it's not concurrency-safe under load.

### Typst template rendered via minijinja before each PDF conversion

The `default.typ` template is loaded into a minijinja `Environment` at startup. Before invoking Pandoc for PDF, the service renders the template with per-request context and writes the result to a temp file, passing that path as `--template` to Pandoc.

**Rationale**: Enables per-request metadata injection (e.g., document title, author) without modifying the Typst source.

### Error handling: AppError enum + IntoResponse

`src/error.rs` defines an `AppError` enum with variants for `InvalidRequest`, `ConversionFailed`, and `InternalError`. Each variant implements `IntoResponse` mapping to 400/422/500 with a JSON body `{"error": "...", "message": "..."}`.

Pandoc stderr is captured and included in `ConversionFailed` message.

### Configuration via figment2 + environment variables only

No config file. All settings are environment variables with defaults. `Config` struct derives `Deserialize` and is constructed via `Figment::from(Env::raw())` at startup, then stored in `AppState`.

### OpenAPI via utoipa + utoipa-axum

Request/response structs derive `ToSchema`. Handlers annotated with `#[utoipa::path(...)]`. Routes registered via `OpenApiRouter`. `split_for_parts()` yields the Axum router and the `OpenApi` object; the latter is served at `/openapi.json` and via Swagger UI at `/swagger`.

## Risks / Trade-offs

- [Pandoc process startup cost] → Acceptable for the expected request volume; no mitigation planned. Revisit if p99 latency is a problem.
- [Temp file disk pressure under high concurrency] → Files are scoped to request lifetime and auto-deleted; risk is minimal.
- [Redis unavailable at runtime with feature enabled] → `REDIS_URL` absence disables Redis at runtime even when the feature is compiled in. Connection failure at startup should degrade gracefully to memory-only (log warning, continue).
- [minijinja template rendering error] → Surface as `InternalError` (500) with the render error message.

## Open Questions

(none — PRD is complete and discussion has resolved all design questions)
