## Why

The service has no container packaging, making deployment and environment reproducibility manual. A Dockerfile enables consistent, self-contained deployments with all runtime dependencies included.

## What Changes

- Add a multi-stage `Dockerfile` that builds the Rust binary and assembles a slim runtime image
- Runtime image includes pinned `typst 0.14.2` and `pandoc 3.9.0.2` binaries downloaded from official releases
- Templates (`templates/`) and filters (`filters/`) are copied into the image
- Default config env vars (`TYPST_TEMPLATE`, `REFERENCE_DOCX`, `LUA_FILTER`) are set to absolute paths inside the image
- Binary is compiled with `--features redis-cache`

## Non-Goals

- No `docker-compose.yml` (out of scope for this change)
- No CI/CD pipeline integration
- No Kubernetes manifests

## Capabilities

### New Capabilities

- `docker-packaging`: Container image that builds and runs md-export with all runtime dependencies (typst, pandoc, templates, filters) self-contained

### Modified Capabilities

(none)

## Impact

- Affected specs: `docker-packaging` (new)
- Affected code: `Dockerfile` (new), `Cargo.toml` (referenced for feature flag)
- New runtime dependencies: `typst 0.14.2`, `pandoc 3.9.0.2` (pinned release binaries)
