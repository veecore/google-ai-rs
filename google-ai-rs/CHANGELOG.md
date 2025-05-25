# Changelog

## 0.1.1 - 2024-02-20

### Added
- `TryIntoContents` trait for validated content creation
- `Tuple` and `Map` type for schema-compliant key/value storage
- `TypedModel` a type-safe model
- Serde integration examples in docs

### Changed
- Improved error messages for schema validation
- GenerativeModel::with_system_instruction and GenerativeModel::with_cloned_instruction take IntoContent instead of &str

### Fixed
- Incorrect total_tokens calculation to include cached content tokens
- Added check for existing for existing schema type before defaulting to "application/json"