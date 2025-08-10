#![allow(unused_imports)]
#![deny(clippy::future_not_send)]

//! Rust client for Google's Generative AI APIs
//!
//! Provides a highly ergonomic, type-safe, and performant interface for
//! interacting with Google's Generative AI services, including Gemini.
//!
//! ## 💡 Highlights
//! - **Minimal Overhead**: The core `Client` is highly optimized, with a small memory footprint and
//!   minimal heap allocations.
//! - **Configurable**: Use optional features to enable TLS backends, JWT authentication,
//!   or reduce dependencies.
//! - **Fluent API**: Builder patterns allow for easy configuration of `GenerativeModel`
//!   parameters like temperature, safety settings, and tools.
//! - **Type-Safe Schemas**: Define your expected response structure with the `AsSchema`
//!   derive macro, ensuring compile-time validation and seamless deserialization.
//! - **Stateful Chat Sessions**: The `Session` struct simplifies building chatbots by
//!   managing conversation history automatically.
//!
//! ## Quickstart
//!
//! A simple example of starting a chat session and streaming a response to the console.
//!
//! ```rust,no_run
//! use google_ai_rs::{Client, GenerativeModel};
//! use std::io::{stdout, Write};
//! use tokio::io::AsyncWriteExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Authenticate the client with an API key
//!     let client = Client::new("YOUR_API_KEY").await?;
//!     let model = client.generative_model("gemini-1.5-pro");
//!     
//!     // Start an interactive chat session
//!     let mut chat = model.start_chat();
//!     
//!     println!("🤖 Initializing chat session...");
//!     print!("👤 ");
//!     
//!     // Create a multi-part prompt
//!     let prompt = "Explain the concept of 'Zero-shot learning' using a simple analogy.";
//!     
//!     // Stream the response and print it to the terminal
//!     let mut stream = chat.stream_send_message(prompt).await?;
//!     
//!     print!("🤖 ");
//!     // Simulate typing indicator
//!     let _ = stdout().flush();
//!     
//!     // Use the `write_to_sync` method to write the streamed response
//!     // to an async writer like `tokio::io::stdout()`
//!     stream.write_to_sync(&mut tokio::io::stdout()).await?;
//!     
//!     println!();
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
pub use client::{Client, SharedClient};
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
fn full_model_name(name: &str) -> std::borrow::Cow<'_, str> {
    if name.contains('/') {
        name.into()
    } else {
        format!("models/{name}").into()
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
