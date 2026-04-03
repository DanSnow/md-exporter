## 1. Store version strings in AppState

- [x] 1.1 Add `pandoc_version: String` and `typst_version: String` fields to `AppState` in `src/main.rs` (store version strings in AppState)
- [x] 1.2 Make `probe_version` in `src/routes/health.rs` `pub(crate)` so it can be called from `main.rs` (reuse existing probe_version function)
- [x] 1.3 In `main()` in `src/main.rs`, call `probe_version` for both `typst_bin` and `pandoc_bin` before constructing `AppState`; return an error from `main()` if either fails — causing the process to exit non-zero (pandoc missing at startup, typst missing at startup)

## 2. Health handler reads from AppState, no spawning

- [x] 2.1 Rewrite `health_handler` in `src/routes/health.rs` to read `state.pandoc_version` and `state.typst_version` from `AppState` instead of calling `probe_version` (health handler reads from AppState, no subprocess spawned on health probe)
- [x] 2.2 Remove the `500` error paths for missing binaries from `health_handler` — those cases are now handled at startup (health endpoint reports service status)
