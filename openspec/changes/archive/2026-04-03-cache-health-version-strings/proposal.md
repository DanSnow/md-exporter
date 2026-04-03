## Why

`GET /health` currently spawns `pandoc --version` and `typst --version` on every call. When deployed on Kubernetes, the liveness/readiness probe polls this endpoint every few seconds, creating a steady stream of unnecessary process forks. The version strings are static for the lifetime of a container — they should be resolved once at startup.

## What Changes

- `GET /health` returns version strings cached at startup rather than probing binaries on each request
- Both `typst --version` and `pandoc --version` are run once during service initialization; their output is stored in `AppState`
- If either binary is missing or not executable at startup, the service fails to start (fast-fail)
- The health endpoint response format is unchanged (`status`, `pandoc_version`, `typst_version`, `cache_backend`)

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `health-check`: version strings are now cached at startup rather than probed on every request; startup fails if binaries are missing

## Impact

- Affected specs: `health-check` (behavior change)
- Affected code: `src/main.rs` (probe at startup, store in AppState), `src/routes/health.rs` (read from AppState instead of spawning), `src/config.rs` (no change)
