## Context

md-exporter currently builds its container image using a multi-stage Dockerfile: a `rust:bookworm` builder stage compiles the binary and downloads Pandoc/Typst tarballs, and a `debian:bookworm-slim` runtime stage assembles the final image. This experiment explores replacing the runtime assembly with the Melange + apko toolchain from Chainguard, keeping the same service behavior but producing a Wolfi-based OCI image locally.

Neither Pandoc nor Typst exist in the Wolfi package repository. Both are available in Alpine edge repos (`pandoc-cli`, `typst`). apko supports mixing Wolfi and Alpine repositories.

## Goals / Non-Goals

**Goals:**

- Build `md-export` Rust binary as a signed apk via Melange
- Assemble a runnable OCI image with `docker run` support via apko
- Pull `pandoc-cli` and `typst` from Alpine edge (no Melange config needed for them)
- Non-root user and writable `/tmp` in the final image

**Non-Goals:**

- CI/CD integration
- cosign signing or SBOM publishing
- Replacing or removing the existing Dockerfile
- Version pinning of pandoc/typst

## Decisions

### Use Alpine edge repos for Pandoc and Typst

**Decision**: Pull `pandoc-cli` and `typst` from Alpine edge instead of packaging them with Melange.

Wolfi has neither package. Melange could wrap their release tarballs, but Alpine edge already ships both — `typst 0.14.2` matches the current pin exactly. Adding Melange configs for binary-only packages with no build step provides no value over a direct Alpine pull.

Alternatives considered:
- **Melange for all three** — maximum ecosystem purity, but Melange configs for pandoc/typst would just curl+extract tarballs, identical to the current Dockerfile approach with extra ceremony.
- **Wolfi-only, skip pandoc/typst** — not viable; they are required for the service to function.

### Use Melange fetch+install for the Rust binary

**Decision**: Use Melange's `fetch` + `runs` pipeline to build `md-export` from source inside the Melange build environment.

This gives the binary proper apk metadata (name, version, architecture) and integrates naturally with apko's package installation flow. The alternative — copying a pre-built binary via apko `paths` — bypasses Melange entirely and defeats the purpose of the experiment.

### apko paths for static assets

**Decision**: Use apko's `paths` directive to copy `templates/` and `filters/` into the image.

These are not installable packages — they are project-specific asset directories. Packaging them as apks via Melange would be over-engineering for a local experiment. `paths` handles this cleanly.

### Shell script over Makefile

**Decision**: Use a single `build-apko.sh` shell script rather than a Makefile.

The build sequence is linear (melange → apko → docker load) with no parallelism or incremental build needs. A shell script is simpler, has no dependency, and is easier to read for someone unfamiliar with the toolchain.

## Risks / Trade-offs

- **Alpine/Wolfi ABI mismatch** → Alpine uses musl libc; Wolfi uses glibc. Pandoc and Typst from Alpine are statically linked, so this is not an issue in practice. The `md-export` binary is built inside Melange's Wolfi environment so it links against glibc correctly.
- **pandoc version drift** → Alpine edge ships `pandoc-cli 3.8.2.1`, one minor behind the current Dockerfile pin of `3.9.0.2`. Lua filters or templates may behave differently. Acceptable for a local experiment; must be verified manually.
- **Melange keyrings** → Melange requires a signing key pair for the apk it produces. The build script must generate a keypair and pass it to both `melange build` and `apko build`. The keypair is ephemeral (experiment-only) and must not be committed.
- **apko image load** → apko outputs a tarball, not a daemon image. The script must run `docker load` after `apko build` to make it available for `docker run`.
