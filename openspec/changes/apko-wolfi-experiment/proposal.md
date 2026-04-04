## Why

Explore the Melange + apko ecosystem as an alternative container build approach for md-exporter. The goal is hands-on experience with the full Chainguard toolchain — Melange to build the Rust binary as a signed apk, and apko to assemble the final image using Wolfi + Alpine packages.

## What Changes

- Add `melange.yaml` to build the `md-export` Rust binary as an apk package
- Add `apko.yaml` to assemble the final OCI image from:
  - Wolfi base
  - `pandoc-cli` from Alpine edge repos
  - `typst` from Alpine edge repos
  - `md-export` apk from Melange
- Add a `Makefile` (or shell script) with commands to run the full local build pipeline
- The image must be runnable via `docker run` with correct non-root user and writable `/tmp`

## Non-Goals

- No CI/CD integration — local experiment only
- No version pinning of pandoc/typst (use whatever Alpine edge ships)
- Not a replacement for the existing Dockerfile — both coexist
- No cosign signing or SBOM publishing in this experiment

## Capabilities

### New Capabilities

- `apko-build`: Local OCI image assembly using Melange + apko with Wolfi/Alpine packages

### Modified Capabilities

(none)

## Impact

- Affected specs: `apko-build` (new)
- Affected code: new files `melange.yaml`, `apko.yaml`, `build-apko.sh` at project root; existing `Dockerfile` untouched
- New build-time dependencies: `melange` CLI, `apko` CLI (local dev tools only)
