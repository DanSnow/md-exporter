# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/DanSnow/md-exporter/releases/tag/v0.1.0) - 2026-04-04

### Added

- *(docker)* add multi-stage Dockerfile for containerized deployment
- init implement

### Other

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
