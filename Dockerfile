# syntax=docker/dockerfile:1

# ── Stage 1: builder ────────────────────────────────────────────────────────
FROM rust:bookworm AS builder

WORKDIR /build

# Download external binaries while dependencies compile
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*

# typst 0.14.2 — x86_64 linux musl (static, no libc dependency)
RUN curl -fsSL "https://github.com/typst/typst/releases/download/v0.14.2/typst-x86_64-unknown-linux-musl.tar.xz" \
    | tar -xJ --strip-components=1 -C /usr/local/bin typst-x86_64-unknown-linux-musl/typst

# pandoc 3.9.0.2 — linux amd64
RUN curl -fsSL "https://github.com/jgm/pandoc/releases/download/3.9.0.2/pandoc-3.9.0.2-linux-amd64.tar.gz" \
    | tar -xz --strip-components=2 -C /usr/local/bin pandoc-3.9.0.2/bin/pandoc

# Cache dependencies layer: copy manifests first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --features redis-cache \
    && rm -rf src

# Build the real binary
COPY src ./src
COPY templates ./templates
COPY filters ./filters
RUN touch src/main.rs \
    && cargo build --release --features redis-cache

# ── Stage 2: runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    dumb-init \
    && rm -rf /var/lib/apt/lists/*

# External binaries
COPY --from=builder /usr/local/bin/typst  /usr/local/bin/typst
COPY --from=builder /usr/local/bin/pandoc /usr/local/bin/pandoc

# Application binary
COPY --from=builder /build/target/release/md-export /usr/local/bin/md-export

# Assets
COPY --from=builder /build/templates /app/templates
COPY --from=builder /build/filters   /app/filters

# Default config — all overridable via env vars
ENV TYPST_TEMPLATE=/app/templates/default.typ
ENV REFERENCE_DOCX=/app/templates/reference.docx
ENV LUA_FILTER=/app/filters/table-auto-width.lua

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 CMD ["curl", "-f", "http://localhost:8080/health"]

ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["/usr/local/bin/md-export"]
