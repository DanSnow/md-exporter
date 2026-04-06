# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3](https://github.com/DanSnow/md-exporter/compare/v0.1.2...v0.1.3) (2026-04-06)


### Bug Fixes

* **pdf:** disable paragraph justification to fix inline code spacing ([3c6798a](https://github.com/DanSnow/md-exporter/commit/3c6798a2377d75b734ae3a06acda43db2babaa8c))

## [0.1.2](https://github.com/DanSnow/md-exporter/compare/v0.1.1...v0.1.2) (2026-04-06)


### Bug Fixes

* **docx:** apply lua filter to docx conversion for auto-width table columns ([813d5ec](https://github.com/DanSnow/md-exporter/commit/813d5ec807ec00ef1f1afc845e62f23e36f871e5))

## [Unreleased]

## [0.1.1](https://github.com/DanSnow/md-exporter/releases/tag/v0.1.1) - 2026-04-04

### Added

- *(docker)* add multi-stage Dockerfile for containerized deployment
- init implement

### Other

- use actions/checkout@v6 in release-plz workflow
- bump version to 0.1.1
- fix release-plz workflow to use release command
- release v0.1.0
- update version
- *(deps)* update docker/login-action action to v4
- *(deps)* update docker/metadata-action action to v6
- *(deps)* update docker/setup-buildx-action action to v4
- fix command
- add release-plz workflow with docker publish
- *(deps)* update docker/build-push-action action to v7 ([#4](https://github.com/DanSnow/md-exporter/pull/4))
- *(deps)* update actions/cache action to v5
- *(deps)* update actions/checkout action to v6
- format
- add renovate.json
- add GitHub Actions workflows and update .gitignore
- add README
- *(docker)* add HEALTHCHECK instruction
- *(health)* cache binary version strings at startup to avoid per-probe spawning
- add license
- add spectra skill
- add prd

## [0.1.0](https://github.com/DanSnow/md-exporter/releases/tag/v0.1.0) - 2026-04-04

### Added

- *(docker)* add multi-stage Dockerfile for containerized deployment
- init implement

### Other

- update version
- *(deps)* update docker/login-action action to v4
- *(deps)* update docker/metadata-action action to v6
- *(deps)* update docker/setup-buildx-action action to v4
- fix command
- add release-plz workflow with docker publish
- *(deps)* update docker/build-push-action action to v7 ([#4](https://github.com/DanSnow/md-exporter/pull/4))
- *(deps)* update actions/cache action to v5
- *(deps)* update actions/checkout action to v6
- format
- add renovate.json
- add GitHub Actions workflows and update .gitignore
- add README
- *(docker)* add HEALTHCHECK instruction
- *(health)* cache binary version strings at startup to avoid per-probe spawning
- add license
- add spectra skill
- add prd
