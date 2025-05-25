// This module exists to help catch trivial bugs statically

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote_each_token, ToTokens, TokenStreamExt as _};
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    str::FromStr,
};
use syn::{spanned::Spanned as _, token::Brace, ExprPath, Ident};

use crate::Context;

macro_rules! quote_each_token_spanned {
    ($span_elem:ident=> $tokens:ident $($tts:tt)*) => {
        let span = $span_elem.span();
        quote::quote_each_token_spanned!($tokens span $($tts)*)
    };
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Default, Debug)]
pub(super) enum Type {
    /// Not specified, should not be used.
    #[default]
    Unspecified,
    /// String type.
    String = 1,
    /// Number type.
    Number = 2,
    /// Integer type.
    Integer = 3,
    /// Boolean type.
    Boolean = 4,
    /// Array type.
    Array = 5,
    /// Object type.
    Object = 6,
}

impl Type {
    pub(super) fn is_compatible_with(self, format: Format) -> bool {
        matches!(
            (self, format),
            (Type::String, Format::Enum)
                | (
                    Type::Number | Type::Integer,
                    Format::Float | Format::Double | Format::Int32 | Format::Int64,
                )
                | (_, Format::Empty)
        )
    }
}

impl FromStr for Type {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(Self::Unspecified),
            "String" => Ok(Self::String),
            "Number" => Ok(Self::Number),
            "Integer" => Ok(Self::Integer),
            "Boolean" => Ok(Self::Boolean),
            "Array" => Ok(Self::Array),
            "Object" => Ok(Self::Object),
            _ => Err("Invalid type"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ty = match self {
            Type::Unspecified => "Unspecified",
            Type::String => "String",
            Type::Number => "Number",
            Type::Integer => "Integer",
            Type::Boolean => "Boolean",
            Type::Array => "Array",
            Type::Object => "Object",
        };
        f.write_str(ty)
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let v = *self as i32;
        v.to_tokens(tokens);
    }
}

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
pub(super) enum Format {
    #[default]
    Empty,
    Float,
    Double,
    Int32,
    Int64,
    Enum,
}

impl FromStr for Format {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            "int32" => Ok(Self::Int32),
            "int64" => Ok(Self::Int64),
            "enum" => Ok(Self::Enum),
            "" => Ok(Self::Empty),
            _ => Err("Invalid format"),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = match self {
            Format::Float => "float",
            Format::Double => "double",
            Format::Int32 => "int32",
            Format::Int64 => "int64",
            Format::Enum => "enum",
            Format::Empty => "",
        };
        f.write_str(format)
    }
}

impl ToTokens for Format {
    fn to_tokens(&self, mut tokens: &mut TokenStream2) {
        let v = self.to_string();
        quote_each_token! {tokens
            #v.to_owned()
        }
    }
}

// TODO: Reduce clone/computation cost... maybe have something
// like borrow which reuses the computation of first
// but then the to_tokens implementation...man... tricks
// are welcome
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum Value<T> {
    Raw(T),
    ReCompute(ExprPath, T),
}

impl<T: ToTokens> ToTokens for Value<T> {
    fn to_tokens(&self, mut tokens: &mut TokenStream2) {
        match self {
            Value::Raw(val) => {
                quote_each_token! {tokens
                    #val.into()
                }
            }
            Value::ReCompute(reval, val) => {
                quote_each_token_spanned! {reval=> tokens
                    #reval(#val.into())
                }
            }
        };
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub(crate) enum BaseSchema {
    Type(syn::Type),
    AsSschema(ExprPath),
    AsSschemaGeneric(ExprPath, syn::Type),
    #[default]
    Empty,
}

impl BaseSchema {
    fn is_some(&self) -> bool {
        !matches!(self, BaseSchema::Empty)
    }
}

impl ToTokens for BaseSchema {
    fn to_tokens(&self, mut tokens: &mut TokenStream2) {
        match self {
            BaseSchema::Type(ty) => {
                quote_each_token_spanned! {ty=> tokens
                    <#ty as AsSchema>::as_schema()
                }
            }
            BaseSchema::AsSschema(as_schema) => {
                quote_each_token_spanned! {as_schema=> tokens
                    #as_schema()
                }
            }
            BaseSchema::AsSschemaGeneric(as_schema_generic, ty) => {
                quote_each_token_spanned! {as_schema_generic=> tokens
                    {
                        let (schema, _): (_, ::std::marker::PhantomData::<#ty>) = #as_schema_generic();
                        schema
                    }
                }
            }
            BaseSchema::Empty => {
                quote_each_token! {tokens
                    Schema {..::std::default::Default::default()}
                }
            }
        };
    }
}

// Most of these can be borrowed but some are created within a function
#[derive(PartialEq, Eq, Debug, Default)]
pub(super) struct Schema {
    // specifables
    pub(super) r#type: Option<Type>,
    pub(super) format: Option<Format>,
    pub(super) description: Option<String>,
    pub(super) nullable: Option<bool>,
    pub(super) max_items: Option<i64>,
    pub(super) min_items: Option<i64>,

    pub(super) r#enum: Vec<Value<String>>,
    pub(super) items: Option<Box<Schema>>,
    pub(super) properties: HashMap<Value<String>, Schema>,
    pub(super) required: Vec<Value<String>>, // TODO: Avoid double computation here. 'required's are from properties
    pub(super) base: BaseSchema,
}

impl ToTokens for Schema {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Let's do little vetting
        self.vet();

        let braces = Brace::default();
        braces.surround(tokens, |mut tokens| {
            let base = &self.base;
            quote_each_token! {tokens
                let mut schema = #base;
            };

            // This macro should be able to simplify the code more
            macro_rules! transfer_properties {
                (vec, $($property:ident)*) => {{
                    $(
                        if !self.$property.is_empty() {
                            let property = &self.$property;
                            quote_each_token! {tokens
                                schema.$property = ::std::vec![#(#property),*];
                            }
                        }
                    )*
                }};
                ($($property:ident)*) => {{
                    $(
                        if let Some(property) = &self.$property {
                            quote_each_token! {tokens
                                schema.$property = #property.into();
                            }
                        }
                    )*
                }};
            }

            transfer_properties! {
                r#type format description nullable min_items max_items
            }

            transfer_properties! {
                vec, required r#enum
            }

            if !self.properties.is_empty() {
                let properties = SetMap {
                    inner: &self.properties,
                    map_var: Ident::new("properties", Span::call_site()),
                    insert: Ident::new("insert", Span::call_site()),
                };

                let properties_len = self.properties.len();

                quote_each_token! (tokens
                    let mut properties = ::std::collections::HashMap::with_capacity(#properties_len);
                    #properties
                    schema.properties = properties;
                );
            }

            if let Some(items) = &self.items {
                quote_each_token! {tokens
                    schema.items = ::std::option::Option::Some(::std::boxed::Box::new(#items));
               }
            }

            // return
            tokens.append(Ident::new("schema", Span::call_site()))
        });
    }
}

impl Schema {
    fn vet(&self) {
        if self.base.is_some() && (!self.properties.is_empty() || !self.required.is_empty()) {
            panic!("Only items (fields and variants) should have BaseSchema")
        }

        if self.properties.len() < self.required.len() {
            panic!("required fields of struct is more than actual struct fields")
        }

        if let Some(r#type) = self.r#type {
            if let Some(format) = self.format {
                if !r#type.is_compatible_with(format) {
                    panic!("incompatible type and format {type} {format}")
                }
            }
        }
    }
}

// This seems hacky.. and hacky things die fast. We'll see though.
// The other method of using [(K, V)].into() so that it's seen as
// a value only works if the from trait is satisfied.
// This one works for any map with the basic insert api
#[derive(Debug)]
struct SetMap<'a, K, V> {
    inner: &'a HashMap<K, V>,
    map_var: Ident,
    insert: Ident,
}

impl<K, V> ToTokens for SetMap<'_, K, V>
where
    K: ToTokens,
    V: ToTokens,
{
    fn to_tokens(&self, mut tokens: &mut TokenStream2) {
        let map_var = &self.map_var;
        let insert = &self.insert;

        for (k, v) in self.inner {
            quote_each_token! {tokens
                #map_var.#insert(#k, #v);
            }
        }
    }
}

pub(super) struct SchemaImpl<'a> {
    pub(super) ctx: &'a Context,
    pub(super) schema: &'a Schema,
}

impl ToTokens for SchemaImpl<'_> {
    fn to_tokens(&self, mut tokens: &mut TokenStream2) {
        let input = &self.ctx.input;
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident = &input.ident;
        let crate_path = &self.ctx.crate_path;
        let schema = &self.schema;

        quote_each_token! {tokens
            #[automatically_derived]
            impl #impl_generics #crate_path::AsSchema for #ident #ty_generics #where_clause {
                fn as_schema() -> #crate_path::Schema {
                    #[allow(unused_imports)]
                    use #crate_path::{Schema, SchemaType};
                    #schema
                }
            }
        };
    }
}

pub(super) struct SchemaImplOwned {
    pub(super) ctx: Context,
    pub(super) schema: Schema,
}

impl ToTokens for SchemaImplOwned {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        SchemaImpl {
            ctx: &self.ctx,
            schema: &self.schema,
        }
        .to_tokens(tokens);
    }
}
