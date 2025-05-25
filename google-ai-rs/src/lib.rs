//! Rust client for Google's Generative AI APIs
//!
//! Provides a type-safe, ergonomic interface for interacting with Google's AI services
//! including Gemini. Features comprehensive API coverage with compile-time schema
//! validation and async/await support.
//!
//! ## Highlights
//! - **Type-Safe API Interactions**: Generated from official Google discovery documents
//! - **Multi-modal Support**: Text, images, and structured data in single requests
//! - **Production-Ready**: Connection pooling, retries, and comprehensive error handling
//!
//! ## Quickstart
//! ```rust,no_run
//! use google_ai_rs::{Client, GenerativeModel};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("API_KEY".into()).await?;
//!     let model = client.generative_model("gemini-pro");
//!     
//!     let response = model.generate_content(
//!         "Explain quantum physics using pirate metaphors"
//!     ).await?;
//!     
//!     println!("{}", response.text());
//!     Ok(())
//! }
//! ```

#![allow(clippy::doc_lazy_continuation)]

pub mod auth;
pub mod chat;
pub mod client;
pub mod content;
pub mod embedding;
pub mod error;
pub mod genai;
pub mod schema;
pub use auth::Auth;
pub use client::Client;
pub use error::Error;
pub use genai::{GenerativeModel, TypedModel, TypedResponse};

pub use crate::proto::Schema;
pub use crate::schema::{AsSchema, Map, MapTrait, SchemaType, Tuple};

pub use content::{
    IntoContent, IntoContents, IntoParts, TryFromCandidates, TryFromContents, TryIntoContent,
    TryIntoContents,
};
pub use proto::{
    part::Data, CachedContent, Candidate, Content, FunctionCall, GenerationConfig, Part, TaskType,
    Tool,
};

extern crate google_ai_schema_derive;

pub use google_ai_schema_derive::AsSchema;

#[cfg(feature = "serde")]
pub use google_ai_schema_derive::AsSchemaWithSerde;

#[doc(hidden)]
pub mod proto;
/// Formats model names to full resource path format
///
/// Ensures model names follow `models/{model}` format.
fn full_model_name(name: &str) -> String {
    if name.contains('/') {
        name.into()
    } else {
        format!("models/{name}")
    }
}

#[test]
fn full_model_name_test() {
    let tests = [
        ("modelName", "models/modelName"),
        ("tunedModels/modelName", "tunedModels/modelName"),
    ];

    for test in tests {
        assert_eq!(full_model_name(test.0), full_model_name(test.1));
    }
}
