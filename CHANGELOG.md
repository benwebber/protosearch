# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-02-28

### Added

- Added `index_phrases` (`bool`) and `index_prefixes` (`IndexPrefixes`) fields to `FieldMapping`.
- Added `boost` (`double`) field to `FieldMapping`.
- Added `Target` message (renamed from `OutputTarget`) with `label` and `json` fields for specifying literal per-target mappings.
- Added error and warning diagnostics.

  Any error aborts compilation.
- Validate field names match conventional naming rules (`W001`).
- Validate `ignore_above`.

  Emit `E001` when the value is less than or equal to zero.
- Validate `position_increment_gap`.

  Emit `E001` for negative values.
- Validate `index_prefixes.min_chars` and `index_prefixes.max_chars`.

  Emit `E001` for negative `min_chars` or `max_chars` outside `0..=20`.
- Emit `W002` when a `target=` label does not match a known target.
- Emit `E002` or `E003` when a `target` entry's `json` is not valid JSON or is not a JSON object.
- Validate plugin parameters (`target=<label>`).

  Previously, unknown parameters were silently ignored. Now, they cause a fatal error.

### Changed

- **BREAKING:** Renamed the top-level extension field from `(protosearch.field)` to `(protosearch.mapping)`. Moved field parameters to the `(protosearch.mapping).field` field.
- **BREAKING:** Replaced the `dynamic`, `index_options`, and `term_vector` string fields with enum types.
- **BREAKING:** Moved output `name` and `target` to `Mapping`.
- Print warnings to standard error. Only return fatal errors to `protoc` using `set_error()`.

### Fixed

- Corrected `fielddata` type from `google.protobuf.Value` to `bool`.
- Corrected `subobjects` type from `string` to `bool`.

## [0.1.0] - 2026-02-26

Initial release.

[Unreleased]: https://github.com/benwebber/protosearch/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/benwebber/protosearch/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/benwebber/protosearch/tree/v0.1.0
