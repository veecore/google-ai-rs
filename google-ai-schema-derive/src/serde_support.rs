use std::ops::{Deref, DerefMut};

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens as _};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned as _, Data,
    DataStruct, DeriveInput, Error, Fields, FieldsNamed, FieldsUnnamed, Lifetime,
    PredicateLifetime, TraitBound, TypeParamBound, WherePredicate,
};

use crate::{attr::SetAttr, Schema, SchemaImpl};

macro_rules! r#try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}

pub(super) fn derive_schema_with_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut ctx = r#try!(Context::new(input));

    let (schema, serde_impl) = r#try!(match ctx.input.data.clone() {
        // Data::Enum(data) => impl_enum(ctx, &data),
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(fields),
            ..
        }) => tuple_struct(&mut ctx, fields),
        _ => Err(Error::new_spanned(
            &ctx.input,
            "AsSchemaWithSerde currently only supports tuples",
        )),
    });

    let schema_impl = SchemaImpl {
        ctx: &ctx,
        schema: &schema,
    }
    .into_token_stream();

    // serde part.

    let delife = syn::Lifetime::new("'__as_schema_de", Span::call_site());
    let mut ctx = ctx.change_to_serde(&delife);

    let (_, ty_generics, where_clause) = ctx.input.generics.split_for_impl();

    let mut ty_generics_where_clause = TokenStream2::new();
    ty_generics.to_tokens(&mut ty_generics_where_clause);
    where_clause.to_tokens(&mut ty_generics_where_clause);

    ctx.input.generics.params.push(parse_quote!(#delife));

    let (impl_generics, _, _) = ctx.input.generics.split_for_impl();

    let ident = &ctx.input.ident;
    let serde = &ctx.serde_path;

    let serde_impl = quote! {
        #[automatically_derived]
        impl #impl_generics #serde::Deserialize<#delife> for #ident #ty_generics_where_clause {
            #[inline(always)]
            fn deserialize<__D>(__deserializer: __D) -> ::std::result::Result<Self, __D::Error>
                where
                    __D: #serde::Deserializer<#delife>,
            {
                #serde_impl
            }
            // TODO: Support in_place
        }
    };

    TokenStream::from(quote! {
        #schema_impl
        #serde_impl
    })
}

struct Context {
    inner: crate::Context,
    serde_path: syn::Path,
}

impl Context {
    fn new(input: DeriveInput) -> Result<Self, Error> {
        let inner = crate::Context::new(input)?;

        let serde_path = SetAttr::find_serde_crate(&inner.input.attrs)?;
        let serde_path = serde_path.unwrap_or_else(|| syn::parse_quote!(::serde));

        Ok(Self { inner, serde_path })
    }

    // Transforms context for Serde compatibility by:
    // 1. Swapping trait bounds from custom schema to Serde
    // 2. Managing lifetime bounds
    // 3. Preserving existing where clauses
    fn change_to_serde(mut self, delife: &Lifetime) -> Self {
        let serde_path = &self.serde_path;
        let new_bound: TraitBound = parse_quote! (#serde_path::Deserialize<#delife>);
        let old_bound = TypeParamBound::Trait(self.inner.trait_bound);

        self.inner.trait_bound = new_bound.clone();

        let mut life_bounds = Punctuated::new();

        // We change the leftover bounds from lib. Since it may visit(and bound) more
        // or less fields than serde, this is not totally correct. To make it totally
        // correct we'll need to be more involved than we'd want to - checking all cond-
        // itions that make serde bound a field. For now, let's keep it minimum.
        self.input
            .generics
            .make_where_clause()
            .predicates
            .iter_mut()
            .for_each(|predicate| {
                if let WherePredicate::Type(ref mut type_predicate) = predicate {
                    type_predicate.bounds.iter_mut().for_each(|b| {
                        if *b == old_bound {
                            *b = TypeParamBound::Trait(new_bound.clone())
                        }
                    })
                }
            });

        if self.has_static {
            // thanks, lib.
            life_bounds.push(Lifetime::new("'static", Span::call_site()));
        } else {
            self.input.generics.lifetimes().for_each(|l| {
                life_bounds.push(l.lifetime.clone());
            });
        }

        if !life_bounds.is_empty() {
            self.input
                .generics
                .make_where_clause()
                .predicates
                .push(WherePredicate::Lifetime(PredicateLifetime {
                    lifetime: delife.clone(),
                    colon_token: syn::Token![:](Span::call_site()),
                    bounds: life_bounds,
                }));
        }

        self
    }
}

impl Deref for Context {
    type Target = crate::Context;

    fn deref(&self) -> &Self::Target {
        #[allow(non_local_definitions)]
        impl DerefMut for Context {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }

        &self.inner
    }
}

fn tuple_struct(
    ctx: &mut Context,
    mut fields: FieldsUnnamed,
) -> Result<(Schema, TokenStream2), Error> {
    /* Core Process:
        1. Convert tuple fields to named fields
        2. Generate Serde attributes with index-based names
        3. Preserve original types with schema metadata
        4. Create helper struct for deserialization
    */

    let (mut field_defs, mut field_names) = (Vec::new(), Vec::new());
    for (i, field) in fields.unnamed.iter_mut().enumerate() {
        // TODO: Maybe giving the freedom to override our lifeless
        // index-name isn't bad?

        let field_name = syn::Ident::new(&format!("field{i}"), field.span());
        let field_ident = i.to_string();

        // We need to remove the attributes meant for us
        // before constructing for serde or `schema` won't
        // be in scope
        let attrs = field
            .attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("schema"));

        let ty = &field.ty;

        // TODO: Handle Default? code can just add that

        // We override rename like this
        // serde will complain on duplicate anyway
        field_defs.push(quote! {
            #(#attrs)*
            #[serde(rename = #field_ident)]
            #field_name: #ty
        });

        // Keep the updates below
        field.ident = Some(field_name.clone());
        field.attrs.extend([
            parse_quote!(#[schema(skip = "false", required = "true", rename = #field_ident)]),
        ]);

        field_names.push(field_name);
    }

    let (_, ty_generics, where_clause) = ctx.input.generics.split_for_impl();
    let serde_path = &ctx.serde_path;
    let helper_ident = format_ident!("__{}Helper", ctx.input.ident);

    let serde_impl = quote! {
        // use crate path
        #[derive(#serde_path::Deserialize)]
        struct #helper_ident #ty_generics #where_clause {
            #(#field_defs,)*
        }

        let helper = #helper_ident::deserialize(__deserializer)?;
        Ok(Self(
            #(helper.#field_names,)*
        ))
    };

    // keep this below so it doesn't change the bounds
    let schema = crate::named_struct(
        ctx,
        &FieldsNamed {
            brace_token: syn::token::Brace {
                span: fields.paren_token.span,
            },
            named: fields.unnamed,
        },
    )?;

    Ok((schema, serde_impl))
}

#[cfg(test)]
mod test {
    use syn::WhereClause;

    use crate::generate_schema;

    use super::*;

    #[test]
    fn serde_path() {
        let test = parse_quote! {
            #[schema(crate_path = "crate")]
            #[serde(crate = "mod_serde")]
            struct S {
                field: Type
            }
        };

        let ctx = Context::new(test).unwrap();
        assert_eq!(ctx.serde_path, parse_quote!(mod_serde))
    }

    #[test]
    fn change_to_serde() {
        struct Test {
            title: &'static str,
            input: DeriveInput,
            where_clause: Option<WhereClause>,
        }

        let tests = [
            Test {
                title: "no life",
                input: parse_quote! {
                    struct S {
                        field: Type
                    }
                },
                where_clause: Some(parse_quote! {where Type: ::serde::Deserialize<'de>}),
            },
            Test {
                title: "9 lives",
                input: parse_quote! {
                    struct S<'a, 'b, 'c> {
                        field: &'a Type,
                        field1: &'b Type1,
                        field2: &'c Type2
                    }
                },
                where_clause: Some(parse_quote! {
                    where
                        &'a Type: ::serde::Deserialize<'de>,
                        &'b Type1: ::serde::Deserialize<'de>,
                        &'c Type2: ::serde::Deserialize<'de>,
                        'de: 'a + 'b + 'c
                }),
            },
            Test {
                title: "'static only needed",
                input: parse_quote! {
                    struct S<'a> {
                        field: &'static Type,
                        field1: &'a Type1
                    }
                },
                where_clause: Some(parse_quote! {
                where
                    &'static Type: ::serde::Deserialize<'de>,
                    &'a Type1: ::serde::Deserialize<'de>,
                    'de: 'static
                }),
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let mut ctx = Context::new(test.input).unwrap();
            _ = generate_schema(&mut ctx);
            let ctx = ctx.change_to_serde(&Lifetime::new("'de", Span::call_site()));

            assert_eq!(ctx.input.generics.where_clause, test.where_clause);
        }
    }
}
