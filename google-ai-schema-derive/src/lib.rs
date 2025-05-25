//! Schema derivation framework for Google AI APIs
//!
//! Provides procedural macros for generating JSON schemas that comply with
//! Google's Generative AI API specifications. Enables type-safe API interactions
//! through compile-time schema validation.
//!
//! ## Key Features
//! - **Schema-GSMA Compliance**: Derive schemas matching Gemini API requirements
//! - **Serde Integration**: Automatic alignment with deserialization
//! - **Validation Rules**: Enforce Google-specific schema constraints at compile time
//! - **Attribute-based Customization**: Fine-tune schema generation through extensive attributes
//! - **Type Safety**: Compile-time validation of schema constraints
//!
//! ## Core Macros
//! - `#[derive(AsSchema)]`: Main derivation macro for schema generation
//! - `#[derive(AsSchemaWithSerde)]`: Enhanced version with deeper Serde integration
//!
//! ## Attribute Reference
//! ### Container Attributes (struct/enum level)
//! - `description`: Overall schema description
//! - `ignore_serde`: Disable serde integration
//! - `rename_all`: Naming convention (e.g., "camelCase", "snake_case")
//! - `rename_all_with`: Custom renaming function
//! - `crate_path`: Custom crate path specification
//! - `nullable`: Mark entire structure as nullable
//!
//! ### Field/Variant Attributes
//! - `description`: Field-specific documentation
//! - `format`: Schema format specification (e.g., "date-time", "email")
//! - `type`: Specific schema type
//! - `as_schema`: Custom schema generation function
//! - `as_schema_generic`: Generic custom schema function
//! - `required`: Force requirement status
//! - `min/max_items`: Array size constraints
//! - `nullable`: Mark item as nullable
//! - `skip`: Exclude field from schema
//!
//! ## Important Notes
//! - **Recursive Types**: Not supported due to JSON Schema limitations
//! - **Serde Integration**: Use `AsSchemaWithSerde` for complex serde representations (e.g with Tuple structs)
//! - **Type-Format Compatibility**: Mismatches like `r#type="String" format="float"` throw compile errors
//! - `rename_all` and `rename_all_with` are mutually exclusive

mod attr;
mod schema;
mod serde_support;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::{cell::LazyCell, collections::HashMap};

use attr::{Attr, TopAttr};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use schema::{BaseSchema, Format, Schema, SchemaImpl, SchemaImplOwned, Value};
use syn::{
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned as _,
    token::{Colon, Comma, Paren},
    Data, DataEnum, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, FieldsUnnamed,
    Path, PredicateType, TraitBound, Type, TypeParamBound, TypeTuple, Variant, WherePredicate,
};

/// Derive macro for AsSchema trait.
///
/// ## Implementation Notes
/// ### 1. Custom Schema Generation
/// **`as_schema`** - Direct schema override:
/// ```rust
/// # mod google_ai_rs {
/// #   pub trait AsSchema { fn as_schema() -> Schema; }
/// #   pub enum SchemaType { Unspecified = 0, String = 1, Number = 2, Integer = 3, Boolean = 4, Array = 5,Object = 6, }
/// #   #[derive(Default)]
/// #   pub struct Schema { pub r#type: i32, pub format: String, pub description: String, pub nullable: bool, pub r#enum: Vec<String>,
/// #   pub items: Option<Box<Schema>>, pub max_items: i64, pub min_items: i64, pub properties: std::collections::HashMap<String, Schema>,
/// #   pub required: Vec<String>, }
/// #   impl AsSchema for String {fn as_schema() -> Schema {Schema {r#type: SchemaType::String as i32, ..Default::default()}}}
/// # }
/// # use google_ai_rs::*;
/// #
/// # use google_ai_schema_derive::AsSchema;
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// struct Timestamp {
///     #[schema(as_schema = "datetime_schema")]
///     millis: i64,
/// }
///
/// /// Simple schema function signature
/// fn datetime_schema() -> Schema {
///     # stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
/// ```
///
/// **`as_schema_generic`** - Handle generic types:
/// ```rust
/// # mod google_ai_rs {
/// #   pub trait AsSchema { fn as_schema() -> Schema; }
/// #   pub enum SchemaType { Unspecified = 0, String = 1, Number = 2, Integer = 3, Boolean = 4, Array = 5,Object = 6, }
/// #   #[derive(Default)]
/// #   pub struct Schema { pub r#type: i32, pub format: String, pub description: String, pub nullable: bool, pub r#enum: Vec<String>,
/// #   pub items: Option<Box<Schema>>, pub max_items: i64, pub min_items: i64, pub properties: std::collections::HashMap<String, Schema>,
/// #   pub required: Vec<String>, }
/// #   impl AsSchema for String {fn as_schema() -> Schema {Schema {r#type: SchemaType::String as i32, ..Default::default()}}}
/// # }
/// # use google_ai_rs::*;
/// #
/// use std::marker::PhantomData;
///
/// struct Wrapper<T> {
///     inner: T,
/// }
///
/// # use google_ai_schema_derive::AsSchema;
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// struct Data {
///     #[schema(as_schema_generic = "wrapper_schema")]
///     field: Wrapper<String>,
/// }
///
/// fn wrapper_schema<T: AsSchema>() -> (Schema, PhantomData<Wrapper<T>>) {
///     # stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
///
/// // NOTE: For enums with struct data:
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// enum E {
///     #[schema(as_schema_generic = "variant1_schema")]
///     Variant {a: String, b: String},
/// }
///
/// // Notice that the return is a tuple
/// fn variant1_schema() -> (Schema, PhantomData<(String, String)>) {
///     # stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
///
/// // Although it is more ideal to use ordinary as_schema in this example.
/// ```
///
/// ### 2. Name Transformation
/// **`rename_all`** vs **`rename_all_with`**:
/// ```rust
/// # mod google_ai_rs {
/// #   pub trait AsSchema { fn as_schema() -> Schema; }
/// #   pub enum SchemaType { Unspecified = 0, String = 1, Number = 2, Integer = 3, Boolean = 4, Array = 5,Object = 6, }
/// #   #[derive(Default)]
/// #   pub struct Schema { pub r#type: i32, pub format: String, pub description: String, pub nullable: bool, pub r#enum: Vec<String>,
/// #   pub items: Option<Box<Schema>>, pub max_items: i64, pub min_items: i64, pub properties: std::collections::HashMap<String, Schema>,
/// #   pub required: Vec<String>, }
/// #   impl AsSchema for String {fn as_schema() -> Schema {Schema {r#type: SchemaType::String as i32, ..Default::default()}}}
/// # }
/// # use google_ai_rs::*;
/// #
/// # use google_ai_schema_derive::AsSchema;
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// #[schema(rename_all = "snake_case")]  // Built-in conventions
/// struct SimpleCase { /* ... */ }
///
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// #[schema(rename_all_with = "reverse_names")]  // Custom function
/// struct CustomCase {
///     field_one: String, // Becomes "eno_dleif"
/// }
///
/// /// Must be deterministic pure function
/// fn reverse_names(s: &str) -> String {
///     s.chars().rev().collect()
/// }
/// ```
///
/// ### 3. Type Handling Nuances
/// **Tuples**:
/// ```rust
/// struct SimpleTuple(u32);          // → Transparent wrapper
/// struct MixedTuple(u32, String);   // → Array with unspecified items
/// struct UniformTuple(u32, u32);    // → Array with item(u32 here) schema
/// ```
/// For more control, use AsSchemaWithSerde.
///
/// **Enums**:
///   - **`Data-less enums`** become string enums
/// ```rust
/// # mod google_ai_rs {
/// #   pub trait AsSchema { fn as_schema() -> Schema; }
/// #   pub enum SchemaType { Unspecified = 0, String = 1, Number = 2, Integer = 3, Boolean = 4, Array = 5,Object = 6, }
/// #   #[derive(Default, PartialEq, Eq, Debug)]
/// #   pub struct Schema { pub r#type: i32, pub format: String, pub description: String, pub nullable: bool, pub r#enum: Vec<String>,
/// #   pub items: Option<Box<Schema>>, pub max_items: i64, pub min_items: i64, pub properties: std::collections::HashMap<String, Schema>,
/// #   pub required: Vec<String>, }
/// #   impl AsSchema for String {fn as_schema() -> Schema {Schema {r#type: SchemaType::String as i32, ..Default::default()}}}
/// # }
/// # use google_ai_rs::*;
/// #
/// #
/// # use google_ai_schema_derive::AsSchema;
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// enum Status {
///     Active,
///     Inactive
/// }
///
/// assert_eq!(
///     Status::as_schema(),
///     Schema {
///         r#type: SchemaType::String as i32,
///         format: "enum".to_owned(),
///         r#enum: ["Active".to_owned(), "Inactive".to_owned()].into(),
///         ..Default::default()
///     }
/// )
/// ```
///
///  - **`Data-containing enums`** become structural objects with all fields unrequired by default. Return matches serde deserialization.
///
/// ```rust
/// # mod google_ai_rs {
/// #   pub trait AsSchema { fn as_schema() -> Schema; }
/// #   pub enum SchemaType { Unspecified = 0, String = 1, Number = 2, Integer = 3, Boolean = 4, Array = 5,Object = 6, }
/// #   #[derive(Default, PartialEq, Eq, Debug)]
/// #   pub struct Schema { pub r#type: i32, pub format: String, pub description: String, pub nullable: bool, pub r#enum: Vec<String>,
/// #   pub items: Option<Box<Schema>>, pub max_items: i64, pub min_items: i64, pub properties: std::collections::HashMap<String, Schema>,
/// #   pub required: Vec<String>, }
/// #   impl AsSchema for String {fn as_schema() -> Schema {Schema {r#type: SchemaType::String as i32, ..Default::default()}}}
/// # }
/// # use google_ai_rs::*;
/// #
/// # use google_ai_schema_derive::AsSchema;
/// #[derive(AsSchema)]
/// # #[schema(crate_path = "google_ai_rs")]
/// enum Response {
///     Success { data: String },
///     Error(String),
/// }
///
/// assert_eq!(
///     Response::as_schema(),
///     Schema {
///         r#type: SchemaType::Object as i32,
///         properties: [
///             (
///                 "Success".to_owned(),
///                 Schema {
///                     r#type: SchemaType::Object as i32,
///                     properties: [("data".to_owned(), String::as_schema())].into(),
///                     required: ["data".to_owned()].into(),
///                     ..Default::default()
///                 }
///             ),
///             ("Error".to_owned(), String::as_schema())
///         ].into(),
///         required: vec![], // None is required by default
///         ..Default::default()
///     }
/// )
/// ```
#[proc_macro_derive(AsSchema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_schema_base(input)
        .map(|si| si.into_token_stream())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn derive_schema_base(input: DeriveInput) -> Result<SchemaImplOwned, Error> {
    let mut ctx = Context::new(input)?;
    let schema = generate_schema(&mut ctx)?;
    Ok(SchemaImplOwned { ctx, schema })
}

/// Hybrid derive macro combining custom schema generation with Serde deserialization
///
/// **This is a specialized, opinionated implementation with several constraints**
///
/// Only supports tuple structs for now
///
/// It represents tuples as objects with positional field names ("0", "1", etc)
/// and derives the corresponding serde::Deserialize for it.
///
/// Adding serde and schema attributes is just as natural as if one added
/// both `#[derive(AsSchame)]` and `#[derive(serde::Deserialize)]`.
///
/// This is to be seen as a minimal-effort hack.
///
/// Adding some serde features may cause issues but you'll know at compile time.
#[proc_macro_derive(AsSchemaWithSerde, attributes(schema, serde))]
pub fn derive_schema_with_serde(input: TokenStream) -> TokenStream {
    crate::serde_support::derive_schema_with_serde(input)
}

struct Context {
    input: DeriveInput,
    trait_bound: TraitBound,
    crate_path: Path,
    top_attr: TopAttr,
    // as big brother, let's help serde_support.
    // It may report false negative because not all type is visited
    has_static: bool,
}

impl Context {
    // Instantiates a new Context fetching the crate_path
    // from the top attr alongside.
    fn new(input: DeriveInput) -> Result<Self, Error> {
        let top_attr = attr::parse_top(&input.attrs)?;
        let crate_path = top_attr
            .crate_path
            .as_ref()
            .map_or_else(|| Ok(parse_quote!(::google_ai_rs)), |c| c.parse())?;

        Ok(Self {
            input,
            trait_bound: parse_quote!(#crate_path::AsSchema),
            crate_path,
            top_attr,
            has_static: false,
        })
    }

    // constrain bounds items type to the #crate::AsSchema
    // trait. It checks for static borrows along the way
    // for use in the serde_support module.
    // It prevents unnecessary double bounds
    fn constrain(&mut self, ty: &Type) -> bool {
        // just add all...
        // we get infinite constrain with recursive types.
        // "it's not a bug, it's a feature".
        let predicate = LazyCell::new(|| -> WherePredicate {
            if !self.has_static {
                // Let's handle static detection raw...

                // FIXME: no, I won't
                self.has_static = ty.to_token_stream().to_string().contains("'static");
            }

            // FIXME: The error is only on "Box" if Q is not AsSchema
            // struct T {
            //     field: Box<Q>
            // }

            // Fix span... parse_quote_spanned won't
            let mut bound = self.trait_bound.clone();
            bound.path.segments.iter_mut().for_each(|p| {
                p.ident.set_span(ty.span());
            });
            if let Some(ref mut l) = bound.path.leading_colon {
                l.spans[0] = ty.span();
                l.spans[1] = ty.span();
            }
            WherePredicate::Type(PredicateType {
                lifetimes: None,
                bounded_ty: ty.clone(),
                colon_token: Colon(Span::call_site()),
                bounds: Punctuated::from_iter([TypeParamBound::Trait(bound)]),
            })
        });

        let predicates = &mut self.input.generics.make_where_clause().predicates;
        if !predicates.iter().any(|p| p.eq(&predicate)) {
            predicates.push(predicate.to_owned());
            true
        } else {
            false
        }
    }
}

fn generate_schema(ctx: &mut Context) -> Result<Schema, Error> {
    match ctx.input.data.clone() {
        Data::Struct(data) => impl_struct(ctx, &data),
        Data::Enum(data) => impl_enum(ctx, &data),
        Data::Union(_) => Err(Error::new_spanned(
            &ctx.input,
            "Unions are not supported by AsSchema derive",
        )),
    }
}

fn impl_struct(ctx: &mut Context, data: &DataStruct) -> Result<Schema, Error> {
    dispatch_struct_fields(ctx, &data.fields)
}

fn dispatch_struct_fields(ctx: &mut Context, fields: &Fields) -> Result<Schema, Error> {
    match fields {
        Fields::Named(fields) => named_struct(ctx, fields),
        Fields::Unnamed(fields) => tuple_struct(ctx, fields),
        Fields::Unit => unit_struct(ctx),
    }
}

fn unit_struct(ctx: &mut Context) -> Result<Schema, Error> {
    let top_attr = &ctx.top_attr;

    Ok(Schema {
        r#type: Some(schema::Type::Object),
        description: top_attr.description.clone(),
        nullable: top_attr.nullable,
        ..Default::default()
    })
}

// Applies several conditions to decide on the representation of input
// - If there's only one element, it's represented transparently and
//   transferable top_attrs are applied if the inner doesn't already
//   exist.
//
// - If
//     the items in the tuple are literally, typically equal, it is
//     represented as an array of this type.
//   else
//      it is represented as array of unspecified items or it errors if
//      it finds any attribute probably meant for us on any field
//
//  with the max_min_items as the length of the tuple
fn tuple_struct(ctx: &mut Context, fields: &FieldsUnnamed) -> Result<Schema, Error> {
    // We can no longer add transparent so as not to break code
    // let's add an opposite of that which represents it as an array
    // This doesn't seem so desired so leave for now
    if fields.unnamed.len() == 1 {
        let top_attr = &ctx.top_attr;

        let inner_ty = &fields.unnamed[0].ty;
        let mut schema_attrs = attr::parse_tuple(
            &fields.unnamed[0].attrs,
            top_attr.ignore_serde.unwrap_or(false),
        )?;

        if schema_attrs.description.is_none() {
            schema_attrs.description = top_attr.description.clone()
        }

        if schema_attrs.nullable.is_none() {
            schema_attrs.nullable = top_attr.nullable
        }

        Ok(generate_item_schema(ctx, &schema_attrs, inner_ty)?)
    } else {
        // Check if they all have the same type. This is trivial.
        // std::string::String is to the compiler String but not
        // here. Fighting that is a losing game.
        let equal_item_ty = fields
            .unnamed
            .iter()
            .try_fold(None, |prev: Option<&Type>, f| {
                if prev.is_none_or(|ty| *ty == f.ty) {
                    Some(Some(&f.ty))
                } else {
                    None
                }
            });

        let item_schema = if let Some(Some(item_ty)) = equal_item_ty {
            // We have too many attr options
            generate_item_schema(ctx, &Attr::default(), item_ty)?
        } else {
            // we used to indescriminately treat as an array of unspecified type
            // we keep that as the default but recommend using AsSchemaWithSerde
            // if we find attributes for us which might show that AsSchemaWithSerde
            // is the man for the job.

            // TODO: Maybe provide a way to not make this cause an unrecoverable error
            fields.unnamed.iter().try_for_each(|f| {
                f.attrs.iter().try_for_each(|attr| {
                    if attr.path().is_ident("schema") {
                        Err(Error::new_spanned(
                            attr,
                            "Consider deriving with AsSchemaWithSerde for more control. \
                               AsSchema derivation represents tuple structs as \
                               an array of unspecified items which doens't support attributes.",
                        ))
                    } else {
                        Ok(())
                    }
                })
            })?;

            Schema {
                r#type: Some(schema::Type::Unspecified),
                ..Default::default()
            }
        };

        let len = Some(fields.unnamed.len() as i64);

        Ok(Schema {
            r#type: Some(schema::Type::Array),
            description: ctx.top_attr.description.clone(),
            max_items: len,
            min_items: len,
            items: Some(item_schema.into()),
            ..Default::default()
        })
    }
}

trait StructItem {
    fn name(&self) -> String;
    fn schema_attrs(&self, top_attr: &TopAttr) -> Result<Attr, Error>;
    fn schema(&self, ctx: &mut Context, schema_attrs: &Attr) -> Result<Schema, Error>;
}

impl<I: StructItem> StructItem for &I {
    fn name(&self) -> String {
        (*self).name()
    }

    fn schema_attrs(&self, top_attr: &TopAttr) -> Result<Attr, Error> {
        (*self).schema_attrs(top_attr)
    }

    fn schema(&self, ctx: &mut Context, schema_attrs: &Attr) -> Result<Schema, Error> {
        (*self).schema(ctx, schema_attrs)
    }
}

fn named_struct_like<I, T>(ctx: &mut Context, items: I, is_enum: bool) -> Result<Schema, Error>
where
    I: IntoIterator<Item = T>,
    T: StructItem,
{
    let rename_all = prepare_rename_all(&ctx.top_attr, is_enum)?;
    let items = items.into_iter();

    let mut properties = HashMap::with_capacity(items.size_hint().0);

    let mut required = Vec::new();

    for item in items {
        let schema_attrs = item.schema_attrs(&ctx.top_attr)?;
        if schema_attrs.skip.unwrap_or_default() {
            continue;
        }

        let original_item_name = item.name();

        let field_name = rename_item(rename_all.as_ref(), &original_item_name, &schema_attrs);

        let nullable = schema_attrs.nullable;
        let required_flag = if nullable.is_some() {
            schema_attrs.required.unwrap_or(false)
        } else {
            schema_attrs.required.unwrap_or(true)
        };

        if required_flag {
            required.push(field_name.clone());
        }

        let field_schema = item.schema(ctx, &schema_attrs)?;

        properties.insert(field_name, field_schema);
    }

    Ok(Schema {
        r#type: Some(schema::Type::Object),
        description: ctx.top_attr.description.clone(),
        nullable: ctx.top_attr.nullable,
        properties,
        required,
        ..Default::default()
    })
}

impl StructItem for Field {
    fn name(&self) -> String {
        self.ident
            .as_ref()
            .expect("Named field missing ident")
            .to_string()
    }

    fn schema_attrs(&self, top_attr: &TopAttr) -> Result<Attr, Error> {
        attr::parse_field(&self.attrs, top_attr.ignore_serde.unwrap_or(false))
    }

    fn schema(&self, ctx: &mut Context, schema_attrs: &Attr) -> Result<Schema, Error> {
        generate_item_schema(ctx, schema_attrs, &self.ty)
    }
}

fn named_struct(ctx: &mut Context, fields: &FieldsNamed) -> Result<Schema, Error> {
    named_struct_like(ctx, &fields.named, !IS_ENUM)
}

impl StructItem for Variant {
    fn name(&self) -> String {
        self.ident.to_string()
    }

    fn schema_attrs(&self, top_attr: &TopAttr) -> Result<Attr, Error> {
        // We treat as an object field
        // Make all fields not required by default
        let mut attr = attr::parse_field(&self.attrs, top_attr.ignore_serde.unwrap_or(false))?;
        if attr.required.is_none() {
            attr.required = Some(false)
        }
        Ok(attr)
    }

    fn schema(&self, ctx: &mut Context, schema_attrs: &Attr) -> Result<Schema, Error> {
        // Detect generators like r#type, and as_schema*. Here, we don't
        // dispatch.
        //
        // This is done to support as_schema_generic
        fn data_type(data: &Fields) -> Type {
            if data.len() == 1 {
                // don't ask
                return data.iter().last().unwrap().ty.clone();
            }
            // rust don't have anonymous structs so we
            // represent the struct-like data as tuple
            let elems: Punctuated<Type, Comma> = data.iter().map(|f| f.ty.clone()).collect();

            let paren_token = if !data.is_empty() {
                match data {
                    Fields::Named(f) => Paren {
                        span: f.brace_token.span,
                    },
                    Fields::Unnamed(f) => f.paren_token,
                    Fields::Unit => Paren::default(),
                }
            } else {
                Paren::default()
            };

            Type::Tuple(TypeTuple { paren_token, elems })
        }

        let mut schema = if schema_attrs.r#type.is_some()
            || schema_attrs.as_schema.is_some()
            || schema_attrs.as_schema_generic.is_some()
        {
            generate_item_schema(ctx, schema_attrs, &data_type(&self.fields))?
        } else {
            dispatch_struct_fields(ctx, &self.fields)?
        };

        // macro is tired of me by now.. lol
        macro_rules! transfer_properties {
            ($($property:ident)*) => {{
                $(if schema.$property.is_none() {
                    schema.$property = schema_attrs.$property.clone()
                })*
            }};
        }
        // We add the top attributes values to the schema
        // if there not filled
        transfer_properties! {
            description nullable max_items min_items
        }

        Ok(schema)
    }
}

// Represents an enum in two ways.
//
// - If there's no data in every variant, it is represented using
//   the enum "api" of the schema subset provided by google.
//
// - Else, it is represented as a struct with each field as the "name"
//   of the variant. This matches the default tag of serde. All field
//   is not required by default so that not all is provided and so maybe
//   at least one will be.
fn impl_enum(ctx: &mut Context, data: &DataEnum) -> Result<Schema, Error> {
    // check if it has data
    let has_data = data.variants.iter().any(|v| !v.fields.is_empty());
    if has_data {
        // replace the topattr so it don't get mixed-up
        let original_top_attr = std::mem::take(&mut ctx.top_attr);

        let mut schema = named_struct_like(ctx, &data.variants, IS_ENUM)?;
        schema.description = original_top_attr.description.clone();

        ctx.top_attr = original_top_attr;
        Ok(schema)
    } else {
        let top_attr = &ctx.top_attr;
        let rename_all = prepare_rename_all(top_attr, IS_ENUM)?;

        let mut variants = Vec::with_capacity(data.variants.len());

        for variant in &data.variants {
            let schema_attrs =
                attr::parse_enum(&variant.attrs, top_attr.ignore_serde.unwrap_or(false))?;

            if schema_attrs.skip.unwrap_or_default() {
                continue;
            }

            let field_name = rename_item(
                rename_all.as_ref(),
                &variant.ident.to_string(),
                &schema_attrs,
            );

            variants.push(field_name);
        }

        Ok(Schema {
            r#type: Some(schema::Type::String),
            format: Some(Format::Enum),
            description: top_attr.description.clone(),
            r#enum: variants,
            ..Default::default()
        })
    }
}

// does constrain
fn generate_item_schema(
    ctx: &mut Context,
    schema_attrs: &Attr,
    item_ty: &Type,
) -> Result<Schema, Error> {
    let description = schema_attrs.description.clone();
    let nullable = schema_attrs.nullable;
    let min_items = schema_attrs.min_items;
    let max_items = schema_attrs.max_items;

    if let Some(ty) = &schema_attrs.r#type {
        let r#type: schema::Type = ty.value().parse().map_err(|m| Error::new(ty.span(), m))?;
        let format = schema_attrs
            .format
            .as_deref()
            .unwrap_or_default()
            .parse()
            .unwrap_or_default();

        // Validate type and format combination
        if !r#type.is_compatible_with(format) {
            return Err(Error::new(
                ty.span(),
                format!("format `{format}` not supported for SchemaType::{type}"),
            ));
        }

        Ok(Schema {
            r#type: Some(r#type),
            format: Some(format),
            description,
            nullable,
            max_items,
            min_items,
            ..Default::default()
        })
    } else {
        let base = if let Some(as_schema) = &schema_attrs.as_schema {
            BaseSchema::AsSschema(as_schema.parse()?)
        } else if let Some(as_schema_generic) = &schema_attrs.as_schema_generic {
            BaseSchema::AsSschemaGeneric(as_schema_generic.parse()?, item_ty.clone())
        } else {
            ctx.constrain(item_ty);
            BaseSchema::Type(item_ty.clone())
        };

        Ok(Schema {
            description,
            nullable,
            max_items,
            min_items,
            base,
            ..Default::default()
        })
    }
}

const IS_ENUM: bool = true;

fn prepare_rename_all(top_attr: &TopAttr, is_enum: bool) -> Result<Option<RenameAll>, Error> {
    if let Some(style) = top_attr.rename_all.as_deref() {
        if let Some(ref rename_all_with) = top_attr.rename_all_with {
            return Err(Error::new(
                rename_all_with.span(), // The whole Attribute should be spanned
                "Schema attributes rename_all and rename_all_with can't be both set.",
            ));
        }

        let rename_all = if is_enum {
            attr::rename_all_variants(style)
        } else {
            attr::rename_all(style)
        };
        Ok(rename_all.map(RenameAll::RenameAll))
    } else if let Some(ref rename_all_with) = top_attr.rename_all_with {
        Ok(Some(RenameAll::RenameWith(rename_all_with.parse()?)))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
enum RenameAll {
    RenameAll(fn(&str) -> String),
    RenameWith(syn::ExprPath),
}

fn rename_item(rename_all: Option<&RenameAll>, item_name: &str, item_attr: &Attr) -> Value<String> {
    macro_rules! or_rename {
        ($f:expr) => {
            if let Some(ref rename) = item_attr.rename {
                Value::Raw(rename.to_string())
            } else {
                $f
            }
        };
    }

    match rename_all {
        Some(RenameAll::RenameAll(rename_all)) => or_rename!(Value::Raw(rename_all(item_name))),
        Some(RenameAll::RenameWith(rename_all_with)) => or_rename!(Value::ReCompute(
            rename_all_with.clone(),
            item_name.to_string()
        )),
        None => or_rename!(Value::Raw(item_name.to_string())),
    }
}

#[cfg(test)]
mod test {
    use syn::WhereClause;

    use super::*;

    #[test]
    fn context_init() {
        struct Test {
            title: &'static str,
            input: DeriveInput,
            crate_path: Path,
            trait_bound: TraitBound,
        }

        let tests = [
            Test {
                title: "crate specified",
                input: parse_quote! {
                    #[schema(crate_path = "crate_path")]
                    struct S {

                    }
                },
                crate_path: parse_quote!(crate_path),
                trait_bound: parse_quote!(crate_path::AsSchema),
            },
            Test {
                title: "crate unspecified",
                input: parse_quote! {
                    struct S {
                    }
                },
                crate_path: parse_quote!(::google_ai_rs),
                trait_bound: parse_quote!(::google_ai_rs::AsSchema),
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let ctx = Context::new(test.input).unwrap();
            assert_eq!(ctx.crate_path, test.crate_path);
            assert_eq!(ctx.trait_bound, test.trait_bound);
            assert!(!ctx.has_static);
        }
    }

    #[test]
    fn context_constrain() {
        struct Test {
            title: &'static str,
            input: DeriveInput,
            where_clause: Option<WhereClause>,
            // we test that we're properly supporting serde by recognizing 'static
            has_static: bool,
        }

        let tests = [
            Test {
                title: "plain static",
                input: parse_quote! {
                    struct S {
                        field: &'static Type
                    }
                },
                where_clause: Some(parse_quote! {where &'static Type: ::google_ai_rs::AsSchema}),
                has_static: true,
            },
            Test {
                title: "no static",
                input: parse_quote! {
                    struct S {
                        field: staticType
                    }
                },
                where_clause: Some(parse_quote! {where staticType: ::google_ai_rs::AsSchema}),
                has_static: false,
            },
            Test {
                title: "generic lifetime",
                input: parse_quote! {
                    struct S<'a> {
                        field: &'a Type,
                    }
                },
                where_clause: Some(parse_quote! {where &'a Type: ::google_ai_rs::AsSchema}),
                has_static: false,
            },
            Test {
                title: "inside static",
                input: parse_quote! {
                    struct S {
                        field: Cow<'static, Type>
                    }
                },
                where_clause: Some(
                    parse_quote! {where Cow<'static, Type>: ::google_ai_rs::AsSchema},
                ),
                has_static: true,
            },
            Test {
                title: "skipped",
                input: parse_quote! {
                    struct S {
                        #[schema(skip)]
                        field: Cow<'static, Type>
                    }
                },
                where_clause: None,
                has_static: false,
            },
            Test {
                title: "skipped (1)",
                input: parse_quote! {
                    struct S {
                        #[schema(r#type = "String")]
                        external: Type
                    }
                },
                where_clause: None,
                has_static: false,
            },
            Test {
                title: "skipped (2)",
                input: parse_quote! {
                    struct S {
                        #[schema(as_schema = "type_as_schema")]
                        external: Type
                    }
                },
                where_clause: None,
                has_static: false,
            },
            Test {
                title: "skipped (3)",
                input: parse_quote! {
                    struct S {
                        #[schema(as_schema_generic = "wrapper_as_schema_generic")]
                        external: Wrapper<Type>
                    }
                },
                where_clause: None,
                has_static: false,
            },
            Test {
                title: "generated and skipped - static false negative", // this is admissibly a bug that I won't fix yet
                input: parse_quote! {
                    struct S {
                        #[schema(r#type = "String")]
                        external: &'static Type
                    }
                },
                where_clause: None,
                has_static: false,
            },
            Test {
                title: "double bound",
                input: parse_quote! {
                    struct S {
                        field: Type,
                        field1: Type
                    }
                },
                where_clause: Some(parse_quote! {where Type: ::google_ai_rs::AsSchema}),
                has_static: false,
            },
            Test {
                title: "double bound exists",
                input: parse_quote! {
                    struct S<T: ::google_ai_rs::AsSchema> {
                        field: T,
                    }
                },
                where_clause: Some(parse_quote! {where T: ::google_ai_rs::AsSchema}),
                has_static: false,
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let mut ctx = Context::new(test.input).unwrap();
            _ = generate_schema(&mut ctx);

            assert_eq!(ctx.input.generics.where_clause, test.where_clause);
            assert_eq!(ctx.has_static, test.has_static);
        }
    }

    #[test]
    fn test_derive_schema() {
        struct Test {
            title: &'static str,
            input: DeriveInput,
            want: Option<Schema>,
            should_fail: bool,
            error_like: Option<Vec<&'static str>>,
        }

        // Test as_schema&_generic
        let tests = [
            Test {
                title: "unit struct",
                input: parse_quote! {
                    #[schema(description = "unit struct")]
                    #[schema(nullable = "false")]
                    struct U;
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Object),
                    description: Some("unit struct".to_owned()),
                    nullable: Some(false),
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "tuple intended for AsSchemaWithSerde",
                input: parse_quote! {
                    #[schema(description = "Represents a radioactive element")]
                    struct T(
                       #[schema(description = "Element name")] String,
                       #[schema(description = "Half life")] f64,
                    );
                },
                want: None,
                should_fail: true,
                error_like: Some(vec!["AsSchemaWithSerde"]),
            },
            Test {
                title: "tuple struct",
                input: parse_quote! {
                    struct T(String, f64);
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Array),
                    items: Some(
                        Schema {
                            r#type: Some(schema::Type::Unspecified),
                            ..Default::default()
                        }
                        .into(),
                    ),
                    max_items: Some(2),
                    min_items: Some(2),
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "enum",
                input: parse_quote! {
                    enum E {
                        Variant1,
                        Variant2,
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::String),
                    format: Some(Format::Enum),
                    r#enum: vec![Value::Raw("Variant1".into()), Value::Raw("Variant2".into())],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "named struct",
                input: parse_quote! {
                    struct S<'a, T, U> {
                        field: Vec<T>,
                        field1: Option<&'a U>,
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Object),
                    properties: [
                        (
                            Value::Raw("field".into()),
                            Schema {
                                base: BaseSchema::Type(parse_quote!(Vec<T>)),
                                ..Default::default()
                            },
                        ),
                        (
                            Value::Raw("field1".into()),
                            Schema {
                                base: BaseSchema::Type(parse_quote!(Option<&'a U>)),
                                ..Default::default()
                            },
                        ),
                    ]
                    .into(),
                    required: vec![Value::Raw("field".into()), Value::Raw("field1".into())],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "rename_all_with",
                input: parse_quote! {
                    #[schema(rename_all_with = "suitcase")]
                    struct S {
                        field: (),
                        field1: (),
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Object),
                    properties: [
                        (
                            Value::ReCompute(parse_quote!(suitcase), "field".into()),
                            Schema {
                                base: BaseSchema::Type(parse_quote!(())),
                                ..Default::default()
                            },
                        ),
                        (
                            Value::ReCompute(parse_quote!(suitcase), "field1".into()),
                            Schema {
                                base: BaseSchema::Type(parse_quote!(())),
                                ..Default::default()
                            },
                        ),
                    ]
                    .into(),
                    required: vec![
                        Value::ReCompute(parse_quote!(suitcase), "field".into()),
                        Value::ReCompute(parse_quote!(suitcase), "field1".into()),
                    ],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "rename_all_with (1)",
                input: parse_quote! {
                    #[schema(rename_all_with = "misc::prettycase")]
                    enum Enum {
                        Variant1,
                        Variant2,
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::String),
                    format: Some(Format::Enum),
                    r#enum: vec![
                        Value::ReCompute(parse_quote!(misc::prettycase), "Variant1".into()),
                        Value::ReCompute(parse_quote!(misc::prettycase), "Variant2".into()),
                    ],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "as_schema",
                input: parse_quote! {
                    struct S {
                        #[schema(as_schema = "concrete::as_schema")]
                        field: Type
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Object),
                    properties: [(
                        Value::Raw("field".into()),
                        Schema {
                            base: BaseSchema::AsSschema(parse_quote!(concrete::as_schema)),
                            ..Default::default()
                        },
                    )]
                    .into(),
                    required: vec![Value::Raw("field".into())],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
            Test {
                title: "as_schema_generic",
                input: parse_quote! {
                    struct S {
                        #[schema(as_schema_generic = "generic::as_schema_generic")]
                        field: Wrapper<Type>
                    }
                },
                want: Some(Schema {
                    r#type: Some(schema::Type::Object),
                    properties: [(
                        Value::Raw("field".into()),
                        Schema {
                            base: BaseSchema::AsSschemaGeneric(
                                parse_quote!(generic::as_schema_generic),
                                parse_quote!(Wrapper<Type>),
                            ),
                            ..Default::default()
                        },
                    )]
                    .into(),
                    required: vec![Value::Raw("field".into())],
                    ..Default::default()
                }),
                should_fail: false,
                error_like: None,
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let derived = derive_schema_base(test.input);

            if test.should_fail {
                match derived {
                    Ok(_) => panic!("test did not fail"),
                    Err(err) => {
                        if let Some(error_like) = test.error_like {
                            let mut matches = false;
                            let err = err.to_string();

                            for like in error_like {
                                matches = matches || err.contains(like)
                            }
                            println!("{err}");
                            assert!(matches);
                        }
                    }
                }
            } else {
                let derived = derived.unwrap_or_else(|err| panic!("test failed: {err}"));
                assert_eq!(derived.schema, test.want.unwrap())
            }
        }
    }
}
