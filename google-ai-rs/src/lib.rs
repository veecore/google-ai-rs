#![allow(clippy::doc_lazy_continuation)]

pub mod auth;
pub mod chat;
pub mod client;
pub mod content;
pub mod embedding;
pub mod error;
pub mod genai;
pub mod proto;
pub mod schema;
pub use auth::Auth;
pub use client::Client;
pub use error::Error;
pub use genai::GenerativeModel;

pub use crate::proto::Schema;
pub use crate::schema::AsSchema;
pub use crate::schema::SchemaType;
pub use proto::CachedContent;
pub use proto::Candidate;
pub use proto::Content;
pub use proto::FunctionCall;
pub use proto::GenerationConfig;
pub use proto::Part;
pub use proto::TaskType;
pub use proto::Tool;

extern crate schema_derive;

pub use schema_derive::*;

/// Formats model names to full resource path format
///
/// Ensures model names follow `models/{model}` format.
fn full_model_name(name: &str) -> String {
    if name.contains('/') {
        name.into()
    } else {
        format!("models/{}", name)
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
