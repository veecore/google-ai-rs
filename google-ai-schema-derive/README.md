[![crates.io](https://img.shields.io/crates/v/google-ai-schema-derive)](https://crates.io/crates/google-ai-schema-derive)
[![Documentation](https://docs.rs/google-ai-schema-derive/badge.svg)](https://docs.rs/google-ai-schema-derive)
[![CI Status](https://github.com/veecore/google-ai-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/veecore/google-ai-rs/actions)

# 🧬 google-ai-schema-derive: Type-Safe Gemini Schemas in Rust

**Generate bulletproof JSON schemas for Google AI with derive macros -  
Because guessing games don't belong in production AI systems!**

```toml
[dependencies]
google-ai-schema-derive = "0.1.1"
```

## ⚡️ 10-Second Schema Generation

```rust
#[derive(AsSchema, Deserialize)]
#[schema(description = "Social media post analysis")]
struct PostAnalysis {
    #[schema(description = "Sentiment score (-1 to 1)")]
    sentiment: f32,
    
    #[schema(rename = "topics", description = "Detected topics")]
    hashtags: Vec<String>,
}

// Automatically generates Gemini-compatible schema:
/*
{
  "type": "object",
  "description": "Social media post analysis",
  "properties": {
    "sentiment": {
      "type": "number",
      "format": "float",
      "description": "Sentiment score (-1 to 1)"
    },
    "topics": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Detected topics"
    }
  },
  "required": ["sentiment", "topics"]
}
*/
```

## 🚀 Why Developers Love This

### 1. Type Safety Meets AI Magic
```rust
#[derive(AsSchema, Deserialize)]
struct FinancialReport {
    #[schema(description = "Year-over-year growth %")]
    yoy_growth: f64,
    
    #[schema(
        description = "Key performance indicators",
        min_items = 3,
        max_items = 5
    )]
    kpis: Vec<String>,
}

// Compile-time validation
```

### 2. Serde Superpowers
```rust
#[derive(AsSchema, Serialize)]
#[schema(rename_all = "camelCase")]
struct UserProfile {
    #[serde(rename = "fullName")] // Mirrored in schema!
    name: String,
    
    #[schema(skip)] // Hidden from schema, kept in Serde
    internal_id: Uuid,
}
```

### 3. Real-World AI Schemas Made Easy
**News Article Analysis:**
```rust
#[derive(AsSchema, Deserialize)]
#[schema(description = "News article metadata extraction")]
struct ArticleMeta {
    #[schema(description = "ISO language code")]
    language: String,
    
    #[schema(
        description = "People/organizations mentioned",
        min_items = 1
    )]
    entities: Vec<String>,
    
    #[schema(as_schema = "date_schema")]
    publish_date: i64,
}

fn date_schema() -> Schema {
    Schema {
        r#type: SchemaType::String as i32,
        ..Default::default()
    }
}
```

## 🛠️ Schema Crafting Toolkit

### Core Attributes Cheat Sheet

**Struct-Level Magic:**
```rust
#[schema(
    description = "E-commerce product listing",
    rename_all = "snake_case",
    nullable   // Whole struct can be null
)]
struct Product {
    // ...
}
```

**Field-Level Control:**
```rust
#[schema(
    description = "Price in USD cents",
    r#type = "Integer",
    format = "int64",
    required  // Force include in required[]
)]
price: u64,

#[schema(
    as_schema = "custom_image_schema",  // Full override
    skip      // Exclude from schema
)]
hero_image: Vec<u8>
```

### Enum Strategies
**Simple Variants → String Enum:**
```rust
#[derive(AsSchema)]
enum ContentRating {
    Safe,
    Mature,
    Explicit
}

// Generates: {"type": "string", "enum": ["Safe", "Mature", "Explicit"]}
```

**Complex Variants → Tagged Union:**
```rust
#[derive(AsSchema)]
enum APIResponse {
    Success { data: String },
    Error { code: u16, message: String }
}

/*
{
  "type": "object",
  "properties": {
    "Success": { /* data schema */ },
    "Error": { /* error schema */ }
  }
}
*/
```

## 🚨 Compliance First

### Enforced Best Practices
```rust
#[derive(AsSchema)]
struct MedicalReport {
    diagnosis: String,
    
    #[schema(r#type = "string", format = "float")] // COMPILE ERROR: Type mismatch
    probability: f32
}
```

### Gemini-Specific Rules
- Type-format compatibility checked
- No recursive types

## 📦 Perfect Pairing

**Works seamlessly with `google-ai-rs` main crate:**
```rust
use google_ai_rs::{AsSchema, TypedModel};

#[derive(AsSchema, Deserialize)]
struct LegalClause {
    #[schema(description = "Clause text in Markdown")]
    text: String,
    #[schema(description = "Applicable jurisdictions")]
    regions: Vec<String>,
}

let model = TypedModel::<LegalClause>::new(&client, "gemini-pro");
let clause = model.generate_content("Generate GDPR data processing clause").await?;
```