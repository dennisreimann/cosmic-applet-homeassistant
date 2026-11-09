# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

The **Unreleased** section accumulates changes as they land; its entries are
used verbatim as release notes when cutting a new release (the section is
then renamed to the new version and emptied).

## [Unreleased]

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
