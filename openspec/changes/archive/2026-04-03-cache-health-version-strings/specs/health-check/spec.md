## MODIFIED Requirements

### Requirement: Health endpoint reports service status

The service SHALL expose `GET /health` returning `200 OK` with a JSON body containing `status`, `pandoc_version`, `typst_version`, and `cache_backend` fields. The `pandoc_version` and `typst_version` values SHALL be resolved once at startup and cached for the lifetime of the process; the handler SHALL NOT spawn any subprocess on each request.

#### Scenario: All binaries available

- **WHEN** `GET /health` is called and the service started successfully
- **THEN** the service returns `200 OK` with `{"status": "ok", "pandoc_version": "3.x.x", "typst_version": "0.x.x", "cache_backend": "memory"}`

#### Scenario: Pandoc missing at startup

- **WHEN** the service starts and the Pandoc binary is not found or not executable
- **THEN** the service SHALL fail to start and exit with a non-zero code before accepting any requests

#### Scenario: Typst missing at startup

- **WHEN** the service starts and the Typst binary is not found or not executable
- **THEN** the service SHALL fail to start and exit with a non-zero code before accepting any requests

#### Scenario: No subprocess spawned on health probe

- **WHEN** `GET /health` is called
- **THEN** the handler SHALL return the cached version strings without spawning any child process
