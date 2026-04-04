## ADDED Requirements

### Requirement: Melange config builds the md-export binary as an apk

The repository SHALL contain a `melange.yaml` at the project root that builds the `md-export` Rust binary from source and packages it as a signed apk. The apk SHALL install the binary to `/usr/bin/md-export` and install `templates/` to `/app/templates/` and `filters/` to `/app/filters/`.

#### Scenario: Melange build produces a valid apk

- **WHEN** `melange build melange.yaml` is executed with a signing keypair
- **THEN** it SHALL produce an `.apk` file for the current architecture containing the `md-export` binary at `/usr/bin/md-export`

#### Scenario: Templates and filters are included in the apk

- **WHEN** the apk produced by Melange is installed
- **THEN** `templates/` SHALL be present at `/app/templates/` and `filters/` SHALL be present at `/app/filters/`

---

### Requirement: apko config assembles the final OCI image

The repository SHALL contain an `apko.yaml` at the project root that assembles a final OCI image from: Wolfi base packages, `pandoc-cli` and `typst` from Alpine edge repositories, and the `md-export` apk produced by Melange.

#### Scenario: apko build produces a loadable OCI tarball

- **WHEN** `apko build apko.yaml` is executed with the local Melange apk repository
- **THEN** it SHALL produce an OCI image tarball that can be loaded with `docker load`

#### Scenario: Image contains all required binaries

- **WHEN** the loaded image is run with `docker run --rm <image> which md-export`
- **THEN** it SHALL exit 0 and print `/usr/bin/md-export`

- **WHEN** the loaded image is run with `docker run --rm <image> which pandoc`
- **THEN** it SHALL exit 0

- **WHEN** the loaded image is run with `docker run --rm <image> which typst`
- **THEN** it SHALL exit 0

---

### Requirement: Image runs as a non-root user

The apko image SHALL configure a non-root system user (uid 65532, matching Chainguard convention) as the runtime user. The service SHALL NOT run as root.

#### Scenario: Container process is non-root

- **WHEN** the container is run with `docker run --rm <image> id`
- **THEN** the output SHALL NOT contain `uid=0`

---

### Requirement: /tmp is writable at runtime

The image SHALL configure `/tmp` as a writable directory accessible to the runtime user, because `md-export` writes temporary files during export operations.

#### Scenario: Service can write to /tmp

- **WHEN** the container is run and an export request is processed
- **THEN** the service SHALL be able to create and write files under `/tmp` without permission errors

---

### Requirement: Build script orchestrates the full local pipeline

The repository SHALL contain a `build-apko.sh` shell script that executes the full pipeline in order: generate signing keypair, run `melange build`, run `apko build`, run `docker load`. After the script completes, the image SHALL be available for `docker run`.

#### Scenario: Single script produces a runnable image

- **WHEN** `./build-apko.sh` is executed on a machine with `melange`, `apko`, and `docker` installed
- **THEN** it SHALL complete without error and the image SHALL be loadable and startable with `docker run -p 8080:8080 <image>`

#### Scenario: Script is idempotent

- **WHEN** `./build-apko.sh` is run a second time
- **THEN** it SHALL succeed and overwrite the previous build artifacts without manual cleanup

---

### Requirement: Environment variable defaults match the existing Dockerfile

The apko image SHALL set the same default environment variables as the current Dockerfile so that the service behaves identically when run with no overrides.

#### Scenario: Default env vars are set

- **WHEN** the container is inspected with `docker inspect`
- **THEN** the environment SHALL include `TYPST_TEMPLATE=/app/templates/default.typ`, `REFERENCE_DOCX=/app/templates/reference.docx`, and `LUA_FILTER=/app/filters/table-auto-width.lua`
