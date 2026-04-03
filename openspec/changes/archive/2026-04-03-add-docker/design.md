## Context

md-export is a Rust/Axum HTTP service that shells out to `typst` and `pandoc` for document conversion. Currently there is no container image — deployers must install Rust, typst, pandoc, and copy templates/filters manually. The service reads all config from environment variables via `figment2`.

## Goals / Non-Goals

**Goals:**

- Produce a working `Dockerfile` that builds and runs md-export with zero host dependencies
- Pin `typst` and `pandoc` to versions matching the development environment
- Compile with `--features redis-cache` so Redis support is available at runtime via `REDIS_URL`
- Set absolute-path ENV defaults for template/filter paths so the service works out-of-the-box

**Non-Goals:**

- `docker-compose.yml`, Kubernetes manifests, or CI/CD pipeline integration
- Multi-platform cross-compilation (linux/arm64 etc.) — single amd64 target
- Distroless or fully static (musl) builds — `debian:bookworm-slim` is sufficient

## Decisions

### Multi-stage build: rust builder + debian:bookworm-slim runtime

Use a two-stage Dockerfile. Stage 1 (`rust:latest`) compiles the binary. Stage 2 (`debian:bookworm-slim`) is the runtime — copy only the binary, downloaded tool binaries, and asset files.

Alternatives considered:
- **Single stage**: Final image would include the full Rust toolchain (~1.5 GB overhead). Rejected.
- **Alpine/musl**: Requires cross-compilation setup and musl-libc linking. pandoc does not ship a musl binary. Rejected.

### Pin typst and pandoc via official release binaries

Download release archives from GitHub releases at known versions (typst 0.14.2, pandoc 3.9.0.2) during the builder stage, then copy the extracted binaries into the runtime stage.

Alternatives considered:
- **apt install**: Version not controllable; Debian bookworm ships an older pandoc. Rejected.
- **Mount at runtime**: Requires host to have correct versions installed, defeating the purpose of containerization. Rejected.

### Templates and filters copied into image at fixed absolute paths

Copy `templates/` to `/app/templates/` and `filters/` to `/app/filters/` in the runtime stage. Set ENV defaults:

```
ENV TYPST_TEMPLATE=/app/templates/default.typ
ENV REFERENCE_DOCX=/app/templates/reference.docx
ENV LUA_FILTER=/app/filters/table-auto-width.lua
```

This makes the service work with zero env var configuration while still allowing overrides.

### Use dumb-init as PID 1

Install `dumb-init` in the runtime stage and use it as the container entrypoint (`ENTRYPOINT ["/usr/bin/dumb-init", "--"]`). This ensures zombie reaping for the `typst` and `pandoc` child processes the service spawns.

Alternatives considered:
- **tini**: equivalent functionality, but `dumb-init` is a single static binary with no dependencies and is available via apt. Acceptable either way.
- **No init (direct binary as PID 1)**: Rust does not reap adopted zombie children. When `typst`/`pandoc` exit, they can remain as zombies until the container restarts. Rejected.

### Compile with --features redis-cache

Build the binary with `cargo build --release --features redis-cache`. Redis is opt-in at runtime via `REDIS_URL` — if not set, the service falls back to memory cache. Compiling it in costs nothing at runtime when unused.

## Risks / Trade-offs

- **typst/pandoc download URLs may change** → Pin to specific GitHub release tag URLs; update manually when upgrading versions.
- **Large builder cache** → Rust compilation is slow but Docker layer caching on `Cargo.toml`/`Cargo.lock` mitigates rebuild time.
- **pandoc release binary size** → pandoc is ~130 MB compressed; final image will be larger than a pure-Rust service. Acceptable trade-off for functionality.
