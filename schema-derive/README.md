# Schema-Derive for Google AI (Rust)

**Type-Safe Schema Generation for Gemini API Interactions**

## Purpose-Built for Google AI

### 1. Gemini Schema Compliance
```rust
#[derive(AsSchema, Deserialize)]
#[schema(description = "Chemical element analysis")]
struct ElementAnalysis {
    #[schema(description = "Element symbol")]
    symbol: String,
    
    #[schema(description = "Atomic mass with units")]
    #[serde(rename = "atomicMass")]
    mass: String,
}
```
Generates JSON schemas that exactly match Gemini's API requirements:
- Automatic `description` propagation
- Google-specific type validations
- Required field enforcement


### 3. Optimized Serde Interop
```rust
#[derive(AsSchema, Serialize)]
#[schema(rename_all = "kebab-case")]
struct AnalysisRequest {
    #[serde(rename = "inputData")] // Mirrored in schema
    input_data: String,
}
```
- Automatic alignment with Serde's `rename`/`skip`


## Core Features

### Schema Attributes
`description` 
`required`
`format`
`rename`
`ignore_serde`
`rename_all`
`nullable`
`type`
`as_schema`
`required`
`skip`

## Usage Example

```rust
#[derive(AsSchema, Deserialize)]
#[schema(rename_all = "camelCase")]
#[schema(description = "API response structure")]
struct GeminiResponse {
    #[schema(description = "Confidence score 0-1")]
    confidence: f64,
    
    #[schema(
        description = "MIME type validated content",
    )]
    content_type: String,
}

// Automatically generates compatible schema:
/*
{
  "type": "object",
  "title": "GeminiResponse",
  "description": "API response structure",
  "properties": {
    "confidence": {
      "type": "number",
      "format": "float"
      "description": "Confidence score 0-1"
    },
    "contentType": {
      "type": "string",
      "description": "MIME type validated content",
    }
  },
  "required": ["confidence", "contentType"]
}
*/
```

## Compliance Notes

### Gemini-Specific Rules
1. **Description Requirements**
```rust
#[derive(AsSchema)]
struct Invalid {
    #[schema(description = "")] // Error: Empty description
    field: String,
}
```


## Limitations

1. **Recursive Structures**
```rust
#[derive(AsSchema)]
struct Node {
    children: Vec<Node>, // Error: Stack overflow
}
```

---

**Part of the Google AI Rust Toolkit**  

*"Finally makes Gemini schema authoring feel like native Rust" - Library Author*
