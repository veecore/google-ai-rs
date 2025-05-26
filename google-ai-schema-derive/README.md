[![crates.io](https://img.shields.io/crates/v/google-ai-rs)](https://crates.io/crates/google-ai-rs)
[![Documentation](https://docs.rs/google-ai-rs/badge.svg)](https://docs.rs/google-ai-rs)
[![CI Status](https://github.com/veecore/google-ai-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/veecore/google-ai-rs/actions)

# 🌟 google_ai_rs: Type-Safe Google AI Interactions for Rustaceans

**Build AI-powered apps with Rust's type system as your guardian angel!**  
Now with 200% more schema validation, Serde magic ✨, and content superpowers.

```toml
[dependencies]
google-ai-rs = { version = "0.1.1", features = ["serde"] } 
```

## 🚀 10-Second Example: Parsed Responses FTW!

```rust
use google_ai_rs::{Client, AsSchema, TypedModel, Map};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(AsSchema, Deserialize)]
struct Recipe {
    name: String,
    ingredients: Vec<String>,
    steps: Vec<Map<HashMap<String, String>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("API_KEY".into()).await?;
    let model = TypedModel::<Recipe>::new(&client, "gemini-2.0-pro");
    
    let recipe = model.generate_content(
        "Give me a vegan lasagna recipe with rocket pesto"
    ).await?;

    println!("🌟 {} ({} ingredients)", recipe.name, recipe.ingredients.len());
    Ok(())
}
```

## 🔥 Killer Features

### 1. Type-Safe Prompts → Structured Responses
```rust
#[derive(AsSchema, Deserialize)]
struct FinancialReport {
    quarter: String,
    revenue: f64,
    // Uses our special Map type for schema-compliant maps!
    metrics: Map<HashMap<String, f64>>,
}

let report = model.generate_content((
    "Generate Q2 financial report for NVIDIA",
    "Include revenue growth and key metrics"
)).await?;

println!("Gross margin: {}", report.metrics.get("gross_margin").unwrap_or_default());
```

### 2. Content Creation Supercharged
Mix and match input types like a boss:

```rust
// Multi-part content with validation
struct UserQuery {
    text: String,
    attachments: Vec<Part>,
}

impl TryIntoContents for UserQuery {
    fn try_into_contents(self) -> Result<Vec<Content>, Error> {
        let mut parts = self.text.into_parts();
        parts.extend(self.attachments);
        Ok(vec![Content::from(parts)])
    }
}

model.generate_content(UserQuery {
    text: "Analyze these product shots".into(),
    attachments: vec![
        Part::blob("image/jpeg", product_image1),
        Part::blob("image/jpeg", product_image2)
    ]
}).await?;
```

### 3. Error Handling That Actually Helps
```rust
match model.generate_content(invalid_input).await {
    Ok(data) => /* Happy path */,
    Err(Error::Service(e)) => eprintln!("Model error: {}", e),
    Err(Error::Net(e)) => retry_logic(),
    // ... and more other error variants
}
```

## 🧠 Real-World Example: Fashion Analytics

```rust
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

#[derive(AsSchemaWithSerde)]
struct PriceInfo(
    #[schema(description = "Price USD")] f32,
    #[schema(description = "In stock")] bool
);

// Get structured data from unstructured input!
let model = client.typed_model::<Map<HashMap<FashionBag, PriceInfo>>>();
let inventory = model.generate_content(
    "List 5 luxury bags from Paris Fashion Week with prices"
).await?;

for (bag, price) in &inventory {
    println!("{} {}: ${}", bag.brand, bag.style, price.0);
}
```

## 🛠️ Flexible Content Creation

### Input Anything
```rust
// Tuple of mixed types
model.generate_content((
    "Translate this document",
    Part::blob("application/pdf", resume_pdf),
    "Keep technical terms in English"
)).await?;

// Automatic conversion for Vecs
let slides = vec![
    Part::blob("image/png", slide1),
    Part::blob("image/png", slide2),
];
model.generate_content(slides).await?;
```

### Output Everything
```rust
// Direct deserialization
#[derive(Deserialize)]
struct Analysis {
    score: f32,
    highlights: Vec<String>,
}

let analysis: Analysis = typed_model.generate_content(text).await?;

// Raw response + parsed data
let TypedResponse { t: data, raw } = model.generate_typed_content(prompt).await?;
println!("Safety ratings: {:?}", raw.safety_ratings);
```

## ⚡️ Performance Meets Safety

- **gRPC core** with async/await
- **Connection pooling** out of the box
- **Schema validation** at compile time

```rust
// Batch processing made easy
client.batch()
    .add_content("Doc1", text1)
    .add_content("Doc2", text2)
    .embed() // Get embeddings for all at once
    .await?;
```

## 🔒 Auth That Fits Your Stack

```rust
// Simple API key
Client::new("your-api-key".into()).await?;

// Full service account
Client::builder()
    .timeout(Duration::from_secs(30))
    .build(Auth::service_account("creds.json").await?)
    .await?;
```

---

**Ready to build AI features that don't keep you up at night?**  
`cargo add google-ai-rs` and sleep better tonight 😴