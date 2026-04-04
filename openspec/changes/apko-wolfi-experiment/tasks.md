## 1. Melange Setup

- [x] 1.1 Install `melange` CLI locally (e.g. `brew install melange` or download from GitHub releases) and verify `melange version` works
- [x] 1.2 Install `apko` CLI locally (e.g. `brew install apko` or download from GitHub releases) and verify `apko version` works
- [x] 1.3 Write `melange.yaml` at project root using Melange fetch+install for the Rust binary (build from source inside Melange, not from a pre-built binary): define a pipeline that runs `cargo build --release` inside the Melange build environment, then installs the binary to `/usr/bin/md-export` — satisfies "Melange config builds the md-export binary as an apk"
- [x] 1.4 Add apko paths for static assets in `melange.yaml`: copy `templates/` to `/app/templates/` and `filters/` to `/app/filters/` as part of the Melange install step

## 2. apko Config

- [x] 2.1 Write `apko.yaml` at project root: declare `wolfi-base` as base, use Alpine edge repos for Pandoc and Typst (add Alpine edge repository URL with its signing key, list `pandoc-cli` and `typst` as packages), reference the local Melange apk repo for `md-export`, configure non-root user (uid 65532) — satisfies "apko config assembles the final OCI image" and "Image runs as a non-root user"
- [x] 2.2 Add `paths` entry in `apko.yaml` for `/tmp` with `permissions: 0777` or owned by the runtime user — satisfies "/tmp is writable at runtime"
- [x] 2.3 Add `environment` block in `apko.yaml` with `TYPST_TEMPLATE`, `REFERENCE_DOCX`, and `LUA_FILTER` set to their default paths — satisfies "Environment variable defaults match the existing Dockerfile"

## 3. Build Script

- [x] 3.1 Write `build-apko.sh` shell script over Makefile at project root (shell script chosen for simplicity — no parallelism needed): generate an ephemeral signing keypair with `melange keygen`, run `melange build melange.yaml --signing-key melange.rsa`, run `apko build apko.yaml md-exporter:apko image.tar --keyring-append melange.rsa.pub`, run `docker load < image.tar` — satisfies "Build script orchestrates the full local pipeline"
- [x] 3.2 Make `build-apko.sh` executable (`chmod +x build-apko.sh`) and add `melange.rsa` and `melange.rsa.pub` to `.gitignore` so the keypair is never committed

## 4. Verification

- [x] 4.1 Run `./build-apko.sh` end-to-end and confirm it exits 0 with the image loaded in Docker
- [x] 4.2 Verify all required binaries are present: run `docker run --rm <image> which md-export`, `which pandoc`, `which typst` — satisfies "Image contains all required binaries"
- [x] 4.3 Verify non-root: run `docker run --rm <image> id` and confirm uid is not 0 — satisfies "Image runs as a non-root user"
- [x] 4.4 Verify writable `/tmp`: run `docker run --rm <image> sh -c "touch /tmp/test && echo ok"` — satisfies "/tmp is writable at runtime"
- [x] 4.5 Run `./build-apko.sh` a second time and confirm it succeeds without manual cleanup — satisfies "Build script is idempotent"
- [x] 4.6 Start the service with `docker run -p 8080:8080 <image>` and confirm `GET /health` returns HTTP 200
