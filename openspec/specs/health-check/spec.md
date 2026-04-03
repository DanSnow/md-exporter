# health-check Specification

## Purpose

TBD - created by archiving change 'build-md-export-service'. Update Purpose after archive.

## Requirements

### Requirement: Health endpoint reports service status

The service SHALL expose `GET /health` returning `200 OK` with a JSON body containing `status`, `pandoc_version`, `typst_version`, and `cache_backend` fields.

#### Scenario: All binaries available

- **WHEN** `GET /health` is called and Pandoc and Typst are executable
- **THEN** the service returns `200 OK` with `{"status": "ok", "pandoc_version": "3.x.x", "typst_version": "0.x.x", "cache_backend": "memory"}`

#### Scenario: Pandoc missing

- **WHEN** `GET /health` is called and the Pandoc binary is not found or not executable
- **THEN** the service returns `500` with `{"status": "error", "message": "pandoc not found"}`

#### Scenario: Typst missing

- **WHEN** `GET /health` is called and the Typst binary is not found or not executable
- **THEN** the service returns `500` with `{"status": "error", "message": "typst not found"}`


<!-- @trace
source: build-md-export-service
updated: 2026-04-03
code:
  - src/converter.rs
  - src/routes/mod.rs
  - templates/default.typ
  - src/cache/memory.rs
  - src/cache/mod.rs
  - filters/table-auto-width.lua
  - src/cache/redis.rs
  - src/routes/export.rs
  - src/routes/openapi.rs
  - bruno/opencollection.yml
  - src/main.rs
  - bruno/Export.yml
  - src/error.rs
  - src/routes/health.rs
  - templates/reference.docx
  - Cargo.toml
  - src/config.rs
-->

---
### Requirement: Health endpoint reports cache backend

The `cache_backend` field in the health response SHALL reflect the active cache backend: `"memory"` when only the in-memory cache is active, `"redis"` when the Redis layer is connected.

#### Scenario: Redis active

- **WHEN** `GET /health` is called with `redis-cache` feature enabled and Redis connected
- **THEN** the response includes `"cache_backend": "redis"`

#### Scenario: Memory only

- **WHEN** `GET /health` is called with only in-memory cache active
- **THEN** the response includes `"cache_backend": "memory"`

<!-- @trace
source: build-md-export-service
updated: 2026-04-03
code:
  - src/converter.rs
  - src/routes/mod.rs
  - templates/default.typ
  - src/cache/memory.rs
  - src/cache/mod.rs
  - filters/table-auto-width.lua
  - src/cache/redis.rs
  - src/routes/export.rs
  - src/routes/openapi.rs
  - bruno/opencollection.yml
  - src/main.rs
  - bruno/Export.yml
  - src/error.rs
  - src/routes/health.rs
  - templates/reference.docx
  - Cargo.toml
  - src/config.rs
-->