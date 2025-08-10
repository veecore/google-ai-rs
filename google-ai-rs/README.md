[![crates.io](https://img.shields.io/crates/v/google-ai-rs)](https://crates.io/crates/google-ai-rs)
[![Documentation](https://docs.rs/google-ai-rs/badge.svg)](https://docs.rs/google-ai-rs)
[![CI Status](https://github.com/veecore/google-ai-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/veecore/google-ai-rs/actions)

# 🌟 `google_ai_rs`: The Type-Safe Generative AI Client for Rust

**A Rust client for Google's Generative AI, built with a focus on type-safety, performance, and developer ergonomics.**
This crate combines a minimal footprint with powerful, **type-safe APIs**, making it ideal for everything from hobby projects to high-performance applications.

```toml
[dependencies]
google-ai-rs = { version = "0.1.2", features = ["serde", "tls-native-roots"] }
````

## 🚀 10-Second Example: Chatbot 💬

Engage in an interactive, stateful conversation with a simple chat session.

```rust
use google_ai_rs::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("YOUR_API_KEY").await?;
    let model = client.generative_model("gemini-1.5-flash");
    let mut chat = model.start_chat();

    println!("🤖 Hello! What can I help you with today?");

    let user_message = "What's the best way to get started with Rust?";
    println!("👤 {}", user_message);
    
    // Send a message and get a response
    let response = chat.send_message(user_message).await?;
    
    let model_response = response.text();
    println!("🤖 {}", model_response);

    // Continue the conversation
    let user_message_2 = "Can you show me a simple 'Hello, World!' example?";
    println!("👤 {}", user_message_2);
    let response_2 = chat.send_message(user_message_2).await?;
    
    println!("🤖 {}", response_2.text());

    Ok(())
}
```

## 🧠 Model Configuration and Builders

Customize your generative model with a fluent builder pattern.
Chain methods to set safety settings, generation parameters, tools, and more.

```rust
use google_ai_rs::{
    genai::{HarmBlockThreshold, HarmCategory, SafetySetting},
    Client,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("YOUR_API_KEY").await?;

    let model = client
        .generative_model("gemini-1.5-flash")
        // Control creativity with temperature
        .temperature(0.9)
        // Ensure a predictable and safe response
        .top_k(10)
        // Add custom safety settings
        .safety_settings([
            SafetySetting::new()
                .harm_category(HarmCategory::HateSpeech)
                .harm_threshold(HarmBlockThreshold::BlockOnlyHigh),
            SafetySetting::new()
                .harm_category(HarmCategory::Harassment)
                .harm_threshold(HarmBlockThreshold::BlockMediumAndAbove),
        ]);

    // Use the configured model
    let response = model.generate_content("Tell me a story.").await?;
    println!("{}", response.text());

    Ok(())
}
```

-----

## 🔥 Key Features

  - **Blazing Fast**: Optimized for minimal overhead. At just **152 bytes** for the core `Client` struct, we've minimized heap allocations and unnecessary dependencies.
  - **Type-Safe Schemas**: Define your response schema using plain Rust structs with the `AsSchema` derive macro. The API ensures you get back exactly what you expect.
  - **Flexible Content**: Send text, images, or a combination of both with a simple, ergonomic API.
  - **Configurable Client**: Control TLS backends, authentication methods, and more with a granular feature flag system.
  - **Chat Session Management**: The `Session` struct handles conversation history and state, making it easy to build interactive chatbots.
  - **Streaming Support**: Get real-time responses with `stream_send_message`, perfect for building responsive UIs.

## 🛠️ Detailed Examples

### 1\. Type-Safe Response Parsing with `Map` and `AsSchema`

The `AsSchema` derive macro, combined with `TypedModel`, gives you fully structured responses with compile-time checks. The `Map` and `Tuple` wrappers allow you to represent non-native Google schema types in a type-safe way.

```rust
use google_ai_rs::{AsSchema, AsSchemaWithSerde, Client, Map};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(AsSchemaWithSerde)]
#[schema(description = "A product's price and availability")]
struct PriceInfo(
    #[schema(description = "Price in USD")] f32,
    #[schema(description = "In stock")] bool,
);

#[derive(AsSchema, Deserialize, PartialEq, Eq, Hash)]
#[schema(description = "High-end fashion bag details")]
struct FashionBag {
    #[schema(description = "Designer brand name")]
    brand: String,
    #[schema(description = "Style category")]
    style: String,
    #[schema(description = "Primary material")]
    material: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("YOUR_API_KEY").await?;

    // Use Map to handle a HashMap, which is not natively supported by the API
    let model = client.typed_model::<Map<HashMap<FashionBag, PriceInfo>>>("gemini-1.5-flash");

    let inventory = model
        .generate_content(
            "List 5 luxury bags from Paris Fashion Week with prices and availability.",
        )
        .await?;

    for (bag, price) in &inventory {
        println!(
            "{} {}: ${} (in stock: {})",
            bag.brand, bag.style, price.0, price.1
        );
    }
    Ok(())
}
```

### 2\. Multi-modal and `TryIntoContents` Input

Easily combine text, images, or other parts in a single prompt. This example shows how to implement `TryIntoContents` for a custom struct to provide clean, validated input.

```rust
use google_ai_rs::{Client, Part, Error, content::TryIntoContents, Content};

struct UserQuery {
    text: String,
    attachments: Vec<Part>,
}

impl TryIntoContents for UserQuery {
    fn try_into_contents(self) -> Result<Vec<Content>, Error> {
        let mut parts = vec![Part::from(self.text)];
        parts.extend(self.attachments);
        
        Ok(vec![Content {
            role: "user".to_string(),
            parts,
        }])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("YOUR_API_KEY").await?;
    let model = client.generative_model("gemini-1.5-flash");
    
    // Load image data from disk
    let product_image = std::fs::read("path/to/product.jpg")?;

    let user_query = UserQuery {
        text: "Analyze this product shot for defects".into(),
        attachments: vec![Part::blob("image/jpeg", product_image)],
    };
    
    let response = model.generate_content(user_query).await?;
    println!("{}", response.text());
    
    Ok(())
}
```

### 3\. Text & Multi-modal Embeddings

Generate vector embeddings for single or multiple pieces of content. This is essential for building semantic search, clustering, or RAG (Retrieval-Augmented Generation) systems.

```rust
use google_ai_rs::{Client, Part};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("YOUR_API_KEY").await?;
    let embedding_model = client.embedding_model("embedding-001");

    // Single text embedding
    let text_embedding = embedding_model.embed_content("Hello, world!").await?;
    println!(
        "Text embedding: {} dimensions",
        text_embedding.embedding.unwrap().values.len()
    );

    // Multi-modal content embedding
    let image_data = std::fs::read("path/to/my_image.jpg")?;
    let multi_modal_embedding = embedding_model
        .embed_content((
            "A description of the image content",
            Part::blob("image/jpeg", image_data),
        ))
        .await?;
    println!(
        "Multi-modal embedding: {} dimensions",
        multi_modal_embedding.embedding.unwrap().values.len()
    );

    // Batch embeddings for efficiency
    let documents = vec!["First document.", "Second document.", "Third document."];
    let batch_response = embedding_model
        .new_batch()
        .add_content("Query for documents")
        .add_content_with_title("Doc1", documents[0])
        .add_content_with_title("Doc2", documents[1])
        .embed()
        .await?;

    for (i, embedding) in batch_response.embeddings.iter().enumerate() {
        println!(
            "Embedding for item {}: {} dimensions",
            i + 1,
            embedding.values.len()
        );
    }

    Ok(())
}
```

---

**Ready to build AI features that don't keep you up at night?**  
`cargo add google-ai-rs` and sleep better tonight 😴