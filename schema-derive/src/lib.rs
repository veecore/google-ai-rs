mod attr;

extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Type,
};

/// # Warning: No Recursive Types!
/// This derive macro generates code for a JSON Schema subset that **does not support recursion**.
/// Attempting to use recursive types (e.g., `struct Node { children: Vec<Node> }`) will cause
/// stack overflows during compilation regardless of indirection.
#[proc_macro_derive(AsSchema, attributes(schema, serde))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let schema_impl = match generate_schema(&input) {
        Ok(tokens) => tokens,
        Err(err) => return err.to_compile_error().into(),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let ident = &input.ident;

    TokenStream::from(quote! {
        #[automatically_derived]
        impl #impl_generics ::google_ai_rs::AsSchema for #ident #ty_generics #where_clause {
            fn as_schema() -> ::google_ai_rs::Schema {
                #schema_impl
            }
        }
    })
}

fn generate_schema(input: &DeriveInput) -> Result<TokenStream2, Error> {
    match &input.data {
        Data::Struct(data) => impl_struct(&input.attrs, data),
        Data::Enum(data) => impl_enum(&input.attrs, data),
        Data::Union(_) => Err(Error::new_spanned(
            input,
            "Unions are not supported by AsSchema derive",
        )),
    }
}

fn impl_struct(attrs: &[Attribute], data: &DataStruct) -> Result<TokenStream2, Error> {
    match &data.fields {
        Fields::Named(fields) => named_struct(attrs, fields),
        Fields::Unnamed(fields) => tuple_struct(attrs, fields),
        Fields::Unit => unit_struct(attrs),
    }
}

fn unit_struct(attrs: &[Attribute]) -> Result<TokenStream2, Error> {
    let description = attr::parse_top(attrs)?.description.unwrap_or_default();
    Ok(quote! {
        ::google_ai_rs::Schema {
            r#type: ::google_ai_rs::SchemaType::Object as i32,
            format: ::std::string::String::new(),
            description: #description.to_string(),
            nullable: true,
            ..::std::default::Default::default()
        }
    })
}

fn tuple_struct(attrs: &[Attribute], fields: &FieldsUnnamed) -> Result<TokenStream2, Error> {
    if fields.unnamed.len() == 1 {
        let inner = &fields.unnamed[0].ty;
        let top_attr = attr::parse_top(attrs)?;
        let description = top_attr.description.unwrap_or_default();
        return Ok(quote! {
            {
                let mut schema = <#inner as ::google_ai_rs::AsSchema>::as_schema();
                schema.description = #description.to_string();
                schema
            }
        });
    }

    // we treat as an array of unspecified type
    let description = attr::parse_top(attrs)?.description.unwrap_or_default();
    let len = fields.unnamed.len();
    Ok(quote! {
        ::google_ai_rs::Schema {
            r#type: ::google_ai_rs::SchemaType::Array as i32,
            description: #description.to_string(),
            items: Some(::std::boxed::Box::new(::google_ai_rs::Schema {
                ..::std::default::Default::default()
            })),
            max_items: #len,
            min_items: #len,
            ..::std::default::Default::default()
        }
    })
}

fn named_struct(attrs: &[Attribute], fields: &FieldsNamed) -> Result<TokenStream2, Error> {
    let top_attr = attr::parse_top(attrs)?;
    let description = top_attr.description.unwrap_or_default();
    let rename_all = if let Some(style) = top_attr.rename_all {
        attr::rename_all(&style).unwrap_or(attr::no_rename)
    } else {
        attr::no_rename
    };

    let mut properties = Vec::with_capacity(fields.named.len());
    let mut required = Vec::new();

    for field in &fields.named {
        let schema_attrs = attr::parse_field(&field.attrs, top_attr.ignore_serde.unwrap_or(false))?;
        if schema_attrs.skip.is_some() {
            continue;
        }

        let field_name = schema_attrs.rename.unwrap_or_else(|| {
            rename_all(
                &field
                    .ident
                    .as_ref()
                    .expect("Named field missing ident")
                    .to_string(),
            )
        });

        let nullable = schema_attrs.nullable;
        let required_flag = if nullable.is_some() {
            schema_attrs.required.unwrap_or(false)
        } else {
            schema_attrs.required.unwrap_or(true)
        };

        if required_flag {
            required.push(field_name.clone());
        }

        let description = schema_attrs.description.unwrap_or_default();
        let nullable = nullable.unwrap_or(false);

        let field_schema = if let Some(ty) = schema_attrs.r#type {
            let format = schema_attrs.format.unwrap_or_default();
            let t = ty.value();
            // are we doing too much? Only if we could warn instead
            match (t.as_str(), format.as_str()) {
                ("Number" | "Integer" , "float" | "double" | "int32" | "int64") => {},
                ("String", "enum") => {},
                (_, "") => {},
                (t, f) => return Err(Error::new(ty.span(), format!("format `{f}` not supported for SchemaType::{t}")))
            };

            let ty = Ident::new(&t, ty.span());

            quote! {
                ::google_ai_rs::Schema {
                    r#type: ::google_ai_rs::SchemaType::#ty as i32, // It'd make sense if we report this
                    format: #format.to_string(),
                    description: #description.to_string(),
                    nullable: #nullable,
                    ..::std::default::Default::default()
                }
            }
        } else {
            let base_schema = generate_base_schema(&field.ty, schema_attrs.as_schema.as_ref().map(|v| &**v));
            let nullable_is_specified = schema_attrs.nullable.is_some();
            quote! {
                {
                    let mut schema = #base_schema;
                    if !#description.is_empty() {
                        schema.description = #description.to_string();
                    }

                    if #nullable_is_specified {
                        schema.nullable = #nullable;
                    }

                    schema
                }
            }
        };

        properties.push(quote! {
            properties.insert(#field_name.to_string(), #field_schema);
        });
    }

    let nullable = top_attr.nullable.is_some();
    let properties_len = properties.len();
    Ok(quote! {
        {
            let mut properties = ::std::collections::HashMap::with_capacity(#properties_len);
            #(#properties)*

            ::google_ai_rs::Schema {
                r#type: ::google_ai_rs::SchemaType::Object as i32,
                description: #description.to_string(),
                nullable: #nullable,
                properties,
                required: ::std::vec![#(#required.into()),*],
                ..::std::default::Default::default()
            }
        }
    })
}

fn impl_enum(attrs: &[Attribute], data: &DataEnum) -> Result<TokenStream2, Error> {
    let top_attr = attr::parse_top(attrs)?;
    let description = top_attr.description.unwrap_or_default();
    let rename_all = if let Some(style) = top_attr.rename_all {
        attr::rename_all_variants(&style).unwrap_or(attr::no_rename)
    } else {
        attr::no_rename
    };

    let mut variants = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        if !variant.fields.is_empty() {
            return Err(Error::new_spanned(
                variant,
                "Enums with data variants are not supported",
            ));
        }

        let schema_attrs =
            attr::parse_enum(&variant.attrs, top_attr.ignore_serde.unwrap_or(false))?;
        if schema_attrs.skip.is_some() {
            continue;
        }

        let field_name = schema_attrs
            .rename
            .unwrap_or_else(|| rename_all(&variant.ident.to_string()));

        variants.push(field_name);
    }

    Ok(quote! {
        ::google_ai_rs::Schema {
            r#type: ::google_ai_rs::SchemaType::String as i32,
            format: "enum".to_string(),
            description: #description.to_string(),
            r#enum: ::std::vec![#(#variants.into()),*],
            ..::std::default::Default::default()
        }
    })
}

fn generate_base_schema(ty: &Type, as_schema: Option<&syn::LitStr>) -> TokenStream2 {
    if let Some(as_schema) = as_schema {
        let f = syn::Ident::new(&as_schema.value(), as_schema.span());
        quote! { #f() } // find nicer way
    } else {
        quote! { <#ty as ::google_ai_rs::AsSchema>::as_schema() }        
    }
}

#[cfg(test)]
mod test {
    use crate::attr::{parse_enum, parse_field, Attr};
    use syn::{parse_quote, Attribute, Data, DataStruct, Fields};

    #[test]
    #[should_panic]
    fn unread_attribute() {
        let attrs = get_fields_attrs(parse_quote!(
            struct A {
                #[attr(I_care_less = "something")]
                field: String,
            }
        ));

        for attr in &attrs[0] {
            if attr.path().is_ident("attr") {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        if ident == "what_I_want" {
                            unimplemented!();
                        }
                        // If we Err here, we won't get the error
                    };
                    return Ok(());
                })
                .unwrap();
            }
        }
    }

    #[test]
    fn struct_and_enum_attributes_validity() {
        struct Test {
            title: &'static str,
            input: syn::DeriveInput,
            should_fail: bool,
            error_like: Option<Vec<&'static str>>,
            is_enum: bool,
        }

        let tests = [
            Test {
                title: "invalid boolean",
                input: parse_quote! {struct S {
                    #[schema(skip = "f")]
                    field: String
                }},
                should_fail: true,
                error_like: Some(vec!["only takes one of", "true", "false"]),
                is_enum: false,
            },
            Test {
                title: "",
                input: parse_quote! {struct S {
                    #[schema(nullable = "false")]
                    field: String
                }},
                should_fail: false,
                error_like: None,
                is_enum: false,
            },
            Test {
                title: "valid no-value boolean",
                input: parse_quote! {struct S {
                    #[schema(skip)]
                    field: String
                }},
                should_fail: false,
                error_like: None,
                is_enum: false,
            },
            Test {
                title: "no argument for argument",
                input: parse_quote! {struct S {
                    #[schema(description)]
                    field: String
                }},
                should_fail: true,
                error_like: Some(vec!["needs argument"]),
                is_enum: false,
            },
            Test {
                title: "unknown attribute",
                input: parse_quote! {struct S {
                    #[schema(unknown = "attribute value")]
                    field: String
                }},
                should_fail: true,
                error_like: Some(vec!["unsupported"]),
                is_enum: false,
            },
            Test {
                title: "unconcerned unknown attribute",
                input: parse_quote! {struct S {
                    #[serde(unknown = "attribute value")] // We don't try to correct what's entirely for serde
                    field: String
                }},
                should_fail: false,
                error_like: None,
                is_enum: false,
            },
            Test {
                title: "concerned 'no argument for argument'",
                input: parse_quote! {struct S {
                    #[serde(rename)]
                    field: String
                }},
                should_fail: true,
                error_like: Some(vec!["needs argument"]),
                is_enum: false,
            },
            Test {
                title: "disallowed attribute",
                input: parse_quote! {
                enum Enum {
                    #[schema(description = "This is variant 1")]
                    Variant1,
                }},
                should_fail: true,
                error_like: Some(vec!["disallowed"]),
                is_enum: true,
            },
            Test {
                title: "disallowed value",
                input: parse_quote! {
                struct S {
                    #[schema(format = "int128")]
                    number: Number
                }},
                should_fail: true,
                error_like: Some(vec!["only takes one of", "float", "double"]),
                is_enum: false,
            },
        ];

        for test in tests {
            let first_field_attrs = &get_fields_attrs(test.input)[0];

            let r = if test.is_enum {
                parse_enum(first_field_attrs, false)
            } else {
                parse_field(first_field_attrs, false)
            };
            println!("title: {}", test.title);
            if test.should_fail {
                match r {
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
            } else if let Err(err) = r {
                panic!("test failed: {err}");
            }
        }
    }

    #[test]
    fn attributes() {
        struct Test {
            title: &'static str,
            input: syn::DeriveInput,
            want: Vec<Attr>,
        }

        let tests = [
            Test {
                title: "basic",
                input: parse_quote! {struct S {
                    #[schema(description = "this is my non-negotiable field", required)]
                    #[schema(rename = "ValuableField")]
                    field: Classified,

                    #[schema(r#type = "String")]
                    time: Time,

                    #[schema(r#type = "Number", format = "float")]
                    number: Number
                }},
                want: vec![
                    Attr {
                        description: Some("this is my non-negotiable field".to_string()),
                        required: Some(true),
                        rename: Some("ValuableField".to_string()),
                        ..Default::default()
                    },
                    Attr {
                        r#type: Some("String".into()),
                        ..Default::default()
                    },
                    Attr {
                        r#type: Some("Number".into()),
                        format: Some("float".to_string()),
                        ..Default::default()
                    },
                ],
            },
            Test {
                title: "serde skip - schema don't",
                input: parse_quote! {struct S {
                    #[schema(description = "description of field field", nullable)]
                    #[serde(skip)]
                    #[schema(skip = "false")]
                    field: Nullable<i32>,
                }},
                want: vec![Attr {
                    description: Some("description of field field".to_string()),
                    skip: Some(false),
                    nullable: Some(true),
                    ..Default::default()
                }],
            },
            Test {
                title: "serde skip and rename",
                input: parse_quote! {struct S {
                    #[serde(rename = "TRUE")]
                    #[serde(skip)]
                    rgb: String,
                }},
                want: vec![Attr {
                    rename: Some("TRUE".to_string()),
                    skip: Some(true),
                    ..Default::default()
                }],
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let fields_attrs = get_fields_attrs(test.input);
            assert_eq!(fields_attrs.len(), test.want.len());

            let mut ith = 0;
            for field_attrs in fields_attrs {
                match parse_field(&field_attrs, false) {
                    Ok(attr) => assert_eq!(&attr, &test.want[ith]),
                    Err(err) => panic!("test failed: {err}"),
                };
                ith += 1;
            }
        }
    }

    fn get_fields_attrs(i: syn::DeriveInput) -> Vec<Vec<Attribute>> {
        let mut out = Vec::new();

        match i.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(f),
                ..
            }) => {
                for f in f.named {
                    out.push(f.attrs);
                }
            }
            Data::Enum(data_enum) => {
                for v in data_enum.variants {
                    out.push(v.attrs)
                }
            }
            _ => unimplemented!("union/unnamed struct not supported"),
        };

        out
    }
}
