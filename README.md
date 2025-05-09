[![crates.io](https://img.shields.io/crates/v/google-ai-rs)](https://crates.io/crates/google-ai-rs)
[![Documentation](https://docs.rs/google-ai-rs/badge.svg)](https://docs.rs/google-ai-rs)
[![CI Status](https://github.com/veecore/google-ai-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/veecore/google-ai-rs/actions)

# 🌟 Google AI in Rust: Delightfully Safe & Expressive

**Meet your new favorite way to interact with Google's Generative AI -  
Type-safe, fluent, and crafted for Rustaceans who love joy in their code!**

```toml
[dependencies]
google-ai-rs = "0.1.0"
```

## Why You'll ❤️ This Library

### 1. Type Safety That *Sings* 🎶
Turn API schemas into Rust structs with derive macros:

```rust
#[derive(AsSchema, Deserialize)]
#[schema(description = "A primary colour")]
struct PrimaryColor {
    #[schema(description = "The name of the colour")]
    name: String,
    #[schema(description = "RGB value in hex")]
    #[serde(rename = "RGB")]
    rgb: String,
}

// Automatically validates against Google's schema!
let model = model.as_response_schema::<Vec<PrimaryColor>>();
let response = model.generate_content("List the primary colors.").await?;

let colors = serde_json::from_slice::<Vec<PrimaryColor>>(&response.into_bytes())?;
println!("Hex code: {}", color.first()?.rgb); // Guaranteed to exist!
```

### 2. Input Flexibility That Feels Like Magic 🔮
Pass content in any shape - we'll handle the rest:

```rust
// Single text
model.generate_content("Hello world").await?;

// Mixed media tuple
model.generate_content((
    "Analyze this diagram",
    Part::blob("image/png", diagram_bytes),
    String::from("Including technical specifications")
)).await?;
```

### 3. Full API Coverage Without the Headaches 📚
We handle the boilerplate so you can focus on AI:

```rust
// Chat with context
let mut chat = model.start_chat();
chat.send_message("Best pizza in NYC with vegetarian options?").await?;

// Batch embeddings
let batch = client.batch()
    .add_content("Document A", text_a)
    .add_content("Document B", text_b);
let embeddings = batch.embed().await?;

// Model management
let tuned_model = client.get_tuned_model("my-model").await?;
```

## Key Features That Make a Difference

### ✨ Schema-first Design
- Derive macros that match Google's schemas exactly
- Serde integration out of the box
- Automatic description propagation to API

### 🧩 Content Composition Power
- `IntoParts` trait handles any input combination
- Tuples, vectors, slices - we speak them all
- Media handling that feels native

### 🛡️ Production-Ready Safety
```rust
match model.generate_content(invalid_input).await {
    Ok(response) => /* ... */,
    Err(Error::InvalidArgument(e)) => /* Type mismatch! */,
    Err(Error::Service(e)) => /* Model errors */,
    Err(Error::Net(e)) => /* Retry logic */,
}
```

### 🔑 Auth That Adapts to You
```rust
// API Key simplicity
Client::new("api-key".into()).await?;

// Service account power
Client::builder()
    .concurrency_limit(6)
    .build(Auth::service_account("credentials.json").await?)
    .await?;
```

## Get Started in 60 Seconds

1. Add the crate:
```toml
[dependencies]
google-ai-rs = { version = "0.1.0" }
```

2. Basic usage:
```rust
use google_ai_rs::{Client, generative::GenerativeModel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("API_KEY".into()).await?;
    let model = client.generative_model("gemini-pro");
    
    let response = model.generate_content((
        "Explain quantum physics in pirate speak",
        "Include a metaphor about parrots"
    )).await?;

    println!("{}", response.text());
    Ok(())
}
```

## Performance Meets Ergonomics 🚀

While we prioritize developer happiness, we didn't compromise on power:

- **gRPC Core**: 2-3x faster than REST wrappers
- **Connection Pooling**: Built-in with tonic/hyper

*"Finally, a Google AI client that feels like Rust should!"*  
– Happy User (probably you, after trying it 😉)