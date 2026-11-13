# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

## [1.2.1] - 2026-11-13

### Fixed

- Dependency refresh closes the Dependabot-class findings (rustls ≥
  0.23.46, quinn-proto ≥ 0.11.15) via `cargo update` + targeted bumps;
  direct dependencies updated to dirs 7 and toml 1.
- libcosmic is now pinned to the verified revision so `cargo update`
  cannot silently move the applet's UI base (see AGENTS.md).

## [1.2.0] - 2026-11-13

### Fixed

- A settings save mid-refresh no longer spawns parallel refresh chains
  (generation-based sequencing; stale results and timers are discarded).
- Failed entity fetches keep the last known values and are listed in the
  popup (`fetch-error`); entity URLs are built via the URL builder, so
  entity IDs are no longer percent-encoded twice.
- Config writes are atomic (tempfile, 0600 before persist); TOML parse
  errors are logged instead of silently defaulting.
- Settings: invalid intervals and save failures are shown and the window
  stays open instead of closing as if saved; a broken config file is
  flagged at startup.
- Release archives now ship the settings binary and icon; `install.sh`
  works from a source checkout and an extracted archive, from any cwd.

## [1.1.0] - 2026-10-30

### Fixed

- Failed fetches (network errors, HTTP errors, invalid responses) now keep
  the last known values and clock instead of wiping them.

## [1.0.0] - 2026-10-29

### Added

- Initial release: native COSMIC panel applet displaying a configurable
  list of [Home Assistant](https://www.home-assistant.io/) entities
  directly in the panel.

[Unreleased]: https://github.com/dennisreimann/cosmic-applet-homeassistant/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/dennisreimann/cosmic-applet-homeassistant/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/dennisreimann/cosmic-applet-homeassistant/releases/tag/v1.0.0
