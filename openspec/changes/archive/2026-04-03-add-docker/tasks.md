## 1. Multi-stage build: rust builder + debian:bookworm-slim runtime

- [x] 1.1 Create `Dockerfile` at project root with a `rust:latest` builder stage; compile with `--features redis-cache` (multi-stage Dockerfile exists, compile with --features redis-cache)
- [x] 1.2 Add `debian:bookworm-slim` runtime stage; copy compiled binary from builder stage (runtime stage is slim)

## 2. Pin typst and pandoc via official release binaries

- [x] 2.1 In the builder stage, download `typst` 0.14.2 from the official GitHub release and extract the binary (typst binary is available at runtime, versions are pinned)
- [x] 2.2 In the builder stage, download `pandoc` 3.9.0.2 from the official GitHub release and extract the binary (pandoc binary is available at runtime, versions are pinned)
- [x] 2.3 Copy both `typst` and `pandoc` binaries into the runtime stage under `/usr/local/bin/` (external binaries are pinned and included)

## 3. Templates and filters copied into image at fixed absolute paths

- [x] 3.1 Copy `templates/` into runtime stage at `/app/templates/` and `filters/` at `/app/filters/` (templates and filters are embedded with absolute path defaults)
- [x] 3.2 Set `ENV TYPST_TEMPLATE=/app/templates/default.typ`, `ENV REFERENCE_DOCX=/app/templates/reference.docx`, `ENV LUA_FILTER=/app/filters/table-auto-width.lua`

## 4. Use dumb-init as PID 1 and port

- [x] 4.1 Install `dumb-init` via apt in the runtime stage (dumb-init is used as PID 1)
- [x] 4.2 Set `ENTRYPOINT ["/usr/bin/dumb-init", "--"]` and `CMD ["/usr/local/bin/md-export"]`; add `EXPOSE 8080` (service listens on port 8080)
- [x] 4.3 Verify the image builds and `GET /health` returns HTTP 200 when run with `-p 8080:8080`
