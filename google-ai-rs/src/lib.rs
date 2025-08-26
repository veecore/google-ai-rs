#![allow(unused_imports)]
#![deny(clippy::future_not_send)]
#![allow(clippy::doc_lazy_continuation)]

//! Rust client for Google's Generative AI APIs
//!
//! Provides a highly ergonomic, type-safe, and performant interface for
//! interacting with Google's Generative AI services, including Gemini.
//!
//! ## 💡 Highlights
//! - **Minimal Overhead**: The core `Client` is tiny and allocation-light.
//! - **Configurable**: TLS, JWT auth, and dependency minimization via features.
//! - **Fluent API**: Builder-style configuration for temperature, safety settings, tools, etc.
//! - **Type-Safe Schemas**: Use `AsSchema` to validate responses at compile-time.
//! - **Stateful Chat**: The `Session` struct handles conversation history for you.
//! - **Multi-Modal Input**: Mix text and images with `Part` or your own `TryIntoContents` impl.
//!
//! ## 🚀 Quickstart (Chat Session)
//!
//! A simple example of starting a chat session and streaming a response.
//!
//! ```rust,no_run
//! use google_ai_rs::{Client, GenerativeModel};
//! use std::io::{stdout, Write};
//! use tokio::io::AsyncWriteExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("YOUR_API_KEY").await?;
//!     let model = client.generative_model("gemini-1.5-pro");
//!
//!     let mut chat = model.start_chat();
//!     println!("🤖 Initializing chat session...");
//!
//!     let prompt = "Explain 'Zero-shot learning' with a simple analogy.";
//!
//!     let mut stream = chat.stream_send_message(prompt).await?;
//!
//!     print!("🤖 ");
//!     let _ = stdout().flush();
//!     stream.write_to_sync(&mut tokio::io::stdout()).await?;
//!
//!     println!();
//!     Ok(())
//! }
//! ```
//!
//! ## 📎 Multi-modal Input with `TryIntoContents`
//!
//! Build your own structs that can be turned into model input. Great for combining text + images.
//!
//! ```rust,no_run
//! use google_ai_rs::{Client, Part, Error, content::TryIntoContents, Content};
//!
//! struct UserQuery {
//!     text: String,
//!     attachments: Vec<Part>,
//! }
//!
//! impl TryIntoContents for UserQuery {
//!     fn try_into_contents(self) -> Result<Vec<Content>, Error> {
//!         let mut parts = vec![Part::from(self.text)];
//!         parts.extend(self.attachments);
//!         Ok(vec![Content { role: "user".into(), parts }])
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("YOUR_API_KEY").await?;
//!     let model = client.generative_model("gemini-1.5-flash");
//!
//!     let product_image = std::fs::read("path/to/product.jpg")?;
//!
//!     let user_query = UserQuery {
//!         text: "Analyze this product shot for defects".into(),
//!         attachments: vec![Part::blob("image/jpeg", product_image)],
//!     };
//!
//!     let response = model.generate_content(user_query).await?;
//!     println!("{}", response.text());
//!     Ok(())
//! }
//! ```
//!
//! ## 🧾 Type-Safe Response Parsing with `AsSchema`
//!
//! Strongly typed schemas ensure you get the structure you expect.
//!
//! To enable type-safe response parsing, turn on the `serde` feature:
//!
//! ```rust,ignore
//! use google_ai_rs::{AsSchema, AsSchemaWithSerde, Client, Map};
//! use serde::Deserialize;
//! use std::collections::HashMap;
//!
//! #[derive(AsSchemaWithSerde)]
//! struct PriceInfo(f32, bool); // (price, in stock)
//!
//! #[derive(AsSchema, Deserialize, PartialEq, Eq, Hash)]
//! struct FashionBag {
//!     brand: String,
//!     style: String,
//!     material: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("YOUR_API_KEY").await?;
//!
//!     let model = client.typed_model::<Map<HashMap<FashionBag, PriceInfo>>>("gemini-1.5-flash");
//!
//!     let inventory = model
//!         .generate_content("List 3 luxury bags with prices and stock status.")
//!         .await?;
//!
//!     for (bag, price) in &inventory {
//!         println!("{} {}: ${} (in stock: {})", bag.brand, bag.style, price.0, price.1);
//!     }
//!     Ok(())
//! }
//! ```

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
