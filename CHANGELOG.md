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

## 0.1.2 (Unreleased)

This release focuses on **performance, configurability, and an improved developer experience**. The core client has been significantly optimized, and new features have been added to streamline common AI tasks.

### 🚀 New Features

* **Optimized Client Architecture**: The core `Client` struct has been reduced in size from **512 bytes to just 152 bytes**. This change, along with a reduction in heap allocations, significantly lowers the memory footprint and improves performance.
* **Modular Features**: Authentication methods (JWT, `auth_update`) are now optional, controlled by feature flags. This allows users to pay only for the dependencies they need, reducing build times and final binary size.
    * `auth_update`: Enables a dedicated mechanism for refreshing authentication credentials.
    * `jwt`: Adds support for JSON Web Token-based authentication.
* **Flexible TLS Backends**: Users can now select their preferred TLS backend via new feature flags, giving them control over their security and performance needs.
    * `tls-ring` (default)
    * `tls-aws-lc`
    * `tls-native-roots` (default)
    * `tls-webpki-roots`
    * `tls-default`: A convenience feature that enables both `tls-ring` and `tls-native-roots`.
* **Thread-Safe `SharedClient`**: Introduced a new `SharedClient` struct that wraps the core client in an `Arc`. This provides a cheaply clonable, thread-safe client, making it easier to manage lifetimes in multi-threaded or long-lived applications.
* **Enhanced Builder Patterns**: Added comprehensive builder methods for `GenerativeModel` and `EmbeddingModel`. This allows for a more ergonomic and readable configuration of model parameters like `temperature`, `top_k`, and `safety_settings`.
* **Schema Derivation Improvements**: The `AsSchema` derive macro now supports **concatenation of multi-line descriptions**. Multiple `#[schema(description = "...")]` attributes can be used on a single item, and empty attributes will render as newlines, similar to standard Rust documentation.

### 🧹 Refactors & Improvements

* Improved documentation for key traits and structs, including `AsSchema`, to better reflect new features and provide clearer examples.
* Refined the `README.md` to highlight the crate's core values: **type-safety, performance, and ergonomics**.
* Standardized builder method names and patterns across the crate for greater consistency.