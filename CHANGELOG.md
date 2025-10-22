# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-10-22

### Changed
- Fix clippy warning: remove redundant closure in history display
- Add "cli" to crates.io categories for better discoverability

## [0.1.1] - 2025-10-22

### Changed
- Use `dirs` crate for cross-platform home directory detection
- Cleaner implementation of `get_data_file()` function

## [0.1.0] - 2025-10-22

### Added
- Initial release
- Medication tracking with add, edit, remove, list commands
- Background daemon with desktop notifications
- Interval safety tracking to prevent accidental overdose
- PRN (as-needed) medication support
- Archive system preserves medication history
- History tracking with adherence metrics
- Command aliases for faster typing
- Critical urgency notifications
- Privacy-focused: local storage with 0600 file permissions (Unix)
- Cross-platform support (Linux, macOS, Windows)
