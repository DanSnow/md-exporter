## Context

`src/routes/health.rs` currently calls `probe_version(bin)` — spawning the binary with `--version` — on every `GET /health` request. In Kubernetes, liveness/readiness probes poll this endpoint every few seconds, producing a continuous stream of `pandoc` and `typst` subprocesses with no benefit: the versions don't change while the container is running.

Current `AppState` fields: `config`, `cache`, `typst_env`, `typst_hash`, `cache_backend`.

## Goals / Non-Goals

**Goals:**
- Run `pandoc --version` and `typst --version` exactly once, at startup
- Store the version strings in `AppState` so the health handler reads them directly
- Fail fast at startup if either binary is missing or not executable

**Non-Goals:**
- Changing the `/health` response schema
- Adding separate liveness/readiness endpoints
- Periodic re-probing after startup

## Decisions

### Store version strings in AppState

Add two fields to `AppState` in `src/main.rs`:
```
pandoc_version: String,
typst_version: String,
```

Probe both binaries in `main()` before constructing `AppState`. If either probe fails, return an error from `main()` — the process exits immediately with a non-zero code, which Kubernetes treats as a failed startup.

Alternatives considered:
- **`Arc<OnceLock<String>>`**: lazy init on first health request. Rejected — first probe still happens under a live request, and startup failure is silent until the first probe call.
- **Keep probing per-request, add a TTL cache**: adds complexity for no benefit. The version is constant for the container lifetime. Rejected.

### Reuse existing probe_version function

Move `probe_version` (currently private in `health.rs`) to be accessible from `main.rs`, or duplicate the logic inline. Since it's a small async helper, the cleanest approach is to make it `pub(crate)` in `health.rs` and call it from `main.rs` during startup.

Alternatives considered:
- **Inline the logic in main.rs**: duplicates ~10 lines. Rejected.
- **Move to a shared module**: overkill for 2 call sites. Rejected.

### Health handler reads from AppState, no spawning

`health_handler` reads `state.pandoc_version` and `state.typst_version` directly. The handler no longer needs to be `async` in practice (no I/O), but keeping it `async` is fine for axum compatibility.

The error paths for missing binaries are removed from the handler — those conditions now cause startup failure instead.

## Risks / Trade-offs

- **Binary replaced at runtime**: if `typst` or `pandoc` is replaced on disk after startup (unlikely in a container), the cached version string would be stale. Acceptable — containers are immutable by design.
- **Startup latency**: two `--version` subprocess calls add ~100ms to startup. Negligible compared to Rust init and Tokio runtime startup.
