## ADDED Requirements

### Requirement: Multi-stage Dockerfile exists

The repository SHALL contain a `Dockerfile` at the project root that uses a multi-stage build: a Rust builder stage and a `debian:bookworm-slim` runtime stage.

#### Scenario: Builder stage compiles the binary

- **WHEN** `docker build` is run
- **THEN** the builder stage SHALL compile the Rust binary with `cargo build --release --features redis-cache`

#### Scenario: Runtime stage is slim

- **WHEN** the final image is built
- **THEN** it SHALL NOT contain the Rust toolchain or build artifacts beyond the compiled binary

### Requirement: External binaries are pinned and included

The runtime image SHALL contain `typst` version 0.14.2 and `pandoc` version 3.9.0.2, downloaded from their official GitHub release pages during the build.

#### Scenario: typst binary is available at runtime

- **WHEN** the container starts and receives an export request requiring typst
- **THEN** the service SHALL invoke the `typst` binary successfully without any host installation

#### Scenario: pandoc binary is available at runtime

- **WHEN** the container starts and receives an export request requiring pandoc
- **THEN** the service SHALL invoke the `pandoc` binary successfully without any host installation

#### Scenario: Versions are pinned

- **WHEN** the Dockerfile is built without cache
- **THEN** it SHALL produce an image with exactly typst 0.14.2 and pandoc 3.9.0.2

### Requirement: Templates and filters are embedded with absolute path defaults

The runtime image SHALL contain `templates/` at `/app/templates/` and `filters/` at `/app/filters/`. The image SHALL set the following environment variable defaults:

- `TYPST_TEMPLATE=/app/templates/default.typ`
- `REFERENCE_DOCX=/app/templates/reference.docx`
- `LUA_FILTER=/app/filters/table-auto-width.lua`

#### Scenario: Service starts with no env overrides

- **WHEN** the container is run with no environment variables set
- **THEN** the service SHALL locate templates and filters at their default absolute paths and start successfully

#### Scenario: Env vars can be overridden

- **WHEN** the container is run with `TYPST_TEMPLATE` set to a custom path (e.g., via a volume mount)
- **THEN** the service SHALL use the overridden path instead of the default

### Requirement: dumb-init is used as PID 1

The runtime image SHALL install `dumb-init` and use it as the container entrypoint so that child processes (`typst`, `pandoc`) are properly reaped and do not become zombies.

#### Scenario: No zombie processes after export

- **WHEN** the service processes an export request that spawns a `typst` or `pandoc` subprocess
- **THEN** the subprocess SHALL be reaped after it exits and SHALL NOT appear as a zombie in the process table

#### Scenario: dumb-init wraps the binary

- **WHEN** the Dockerfile is inspected
- **THEN** `ENTRYPOINT` SHALL reference `dumb-init` as the first element before the `md-export` binary

### Requirement: Service listens on port 8080

The runtime image SHALL expose port 8080 and the service SHALL bind to `0.0.0.0:8080` by default.

#### Scenario: Port is exposed

- **WHEN** the Dockerfile is inspected
- **THEN** it SHALL contain `EXPOSE 8080`

#### Scenario: Service is reachable on 8080

- **WHEN** the container is run with `-p 8080:8080`
- **THEN** `GET /health` SHALL return HTTP 200
