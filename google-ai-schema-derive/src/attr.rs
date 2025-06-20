//! Schema attribute parsing utilities
//!
//! Handles conversion between Rust types and OpenAPI schemas with support for:
//! - Field renaming strategies (camelCase, snake_case, etc)
//! - Serde attribute compatibility
//! - Type customization via attributes
//!
//! Serde-compatible field renaming strategies
//!
//! Supported naming conventions:
//! - camelCase
//! - snake_case
//! - lowercase
//! - UPPERCASE  
//! - PascalCase
//! - SCREAMING_SNAKE_CASE
//! - kebab-case
//! - SCREAMING-KEBAB-CASE
//!
//! By default, integrates with Serde's rename attributes. Disable with
//! `ignore_serde` in struct attributes.

use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use proc_macro2::Span;
use syn::{Attribute, Error};

/// Top-level type attributes for schema generation
#[derive(Default)]
pub(crate) struct TopAttr {
    pub(crate) description: Option<String>,
    pub(crate) ignore_serde: Option<bool>,
    pub(crate) rename_all: Option<String>,
    pub(crate) rename_all_with: Option<LitStr>,
    pub(crate) crate_path: Option<LitStr>,
    pub(crate) nullable: Option<bool>,
}

pub(crate) fn parse_top(attrs: &[Attribute]) -> Result<TopAttr, Error> {
    parse_top_item::<0>(attrs, None)
}

fn parse_top_item<const N: usize>(
    attrs: &[Attribute],
    disallow: Option<[&'static str; N]>,
) -> Result<TopAttr, Error> {
    let mut want = SetAttributes::var([
        "description",
        "ignore_serde",
        "rename_all",
        "rename_all_with",
        "crate_path",
        "nullable",
    ])
    .allow_bool(["nullable", "ignore_serde"])
    .only_allow_one_of("rename_all", case::SUPPORTED)
    .disallow(disallow);

    want.get_attrs(attrs, "schema")?;

    let description = want.extract("description")?;
    let ignore_serde = want.extract_bool("ignore_serde")?;
    let mut rename_all = want.extract("rename_all")?;
    let rename_all_with = want.extract_literal("rename_all_with").map(Into::into);
    let crate_path = want.extract_literal("crate_path").map(Into::into);
    let nullable = want.extract_bool("nullable")?;

    if ignore_serde.is_none() && rename_all.is_none() {
        // let's use serde's rename
        want = want.re_var(["rename_all"]);
        want.find_attrs(attrs, "serde")?;
        rename_all = want.extract("rename_all")?;
    }

    Ok(TopAttr {
        description,
        rename_all,
        rename_all_with,
        nullable,
        crate_path,
        ignore_serde,
    })
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Attr {
    pub(crate) description: Option<String>,
    pub(crate) format: Option<String>,
    pub(crate) r#type: Option<LitStr>,
    pub(crate) as_schema: Option<LitStr>,
    pub(crate) as_schema_generic: Option<LitStr>,
    pub(crate) rename: Option<String>,
    pub(crate) required: Option<bool>,
    pub(crate) min_items: Option<i64>,
    pub(crate) max_items: Option<i64>,
    pub(crate) nullable: Option<bool>,
    pub(crate) skip: Option<bool>,
}

pub(crate) struct LitStr(syn::LitStr);

impl Debug for LitStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.value())
    }
}

impl PartialEq for LitStr {
    fn eq(&self, other: &Self) -> bool {
        self.0.value() == other.0.value()
    }
}

impl From<syn::LitStr> for LitStr {
    fn from(value: syn::LitStr) -> Self {
        LitStr(value)
    }
}

impl From<&str> for LitStr {
    fn from(value: &str) -> Self {
        syn::LitStr::new(value, Span::call_site()).into()
    }
}

impl Deref for LitStr {
    type Target = syn::LitStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn parse_field(attrs: &[Attribute], ignore_serde: bool) -> Result<Attr, Error> {
    parse_item::<0>(attrs, ignore_serde, None)
}

pub(crate) fn parse_enum(attrs: &[Attribute], ignore_serde: bool) -> Result<Attr, Error> {
    parse_item(
        attrs,
        ignore_serde,
        Some([
            "description",
            "format",
            "r#type",
            "as_schema",
            "as_schema_generic",
            "min_items",
            "max_items",
            "required",
            "nullable",
        ]),
    )
}

pub(crate) fn parse_tuple(attrs: &[Attribute], ignore_serde: bool) -> Result<Attr, Error> {
    parse_item(attrs, ignore_serde, Some(["rename"]))
}

fn parse_item<const N: usize>(
    attrs: &[Attribute],
    ignore_serde: bool,
    disallow: Option<[&'static str; N]>,
) -> Result<Attr, Error> {
    let mut want = SetAttributes::var([
        "description",
        "format",
        "r#type",
        "as_schema",
        "as_schema_generic",
        "rename",
        "required",
        "min_items",
        "max_items",
        "nullable",
        "skip",
    ])
    .allow_bool(["required", "nullable", "skip"])
    .only_allow_one_of(
        "r#type",
        [
            "Unspecified",
            "String",
            "Number",
            "Integer",
            "Boolean",
            "Array",
            "Object",
        ],
    )
    .only_allow_one_of("format", ["float", "double", "int32", "int64", "enum"])
    .disallow(disallow);

    want.get_attrs(attrs, "schema")?;

    let description = want.extract("description")?;
    let format = want.extract("format")?;
    let r#type = want.extract_literal("r#type").map(Into::into);
    let as_schema = want.extract_literal("as_schema").map(Into::into);
    let as_schema_generic = want.extract_literal("as_schema_generic").map(Into::into);
    let mut rename = want.extract("rename")?;
    let required = want.extract_bool("required")?;
    let min_items = want.extract_int("min_items")?;
    let max_items = want.extract_int("max_items")?;
    let nullable = want.extract_bool("nullable")?;
    let mut skip = want.extract_bool("skip")?;

    if !ignore_serde {
        // We should do it once or want will change before we get down
        // and is_disallowed will be false

        let get_rename = rename.is_none() && !want.is_disallowed("rename");
        let get_skip = skip.is_none() && !want.is_disallowed("skip");

        if get_rename || get_skip {
            want = want.re_var(["rename", "skip"]).allow_bool(["skip"]);

            want.find_attrs(attrs, "serde")?;
            if get_rename {
                rename = want.extract("rename")?;
            }

            if get_skip {
                skip = want.extract_bool("skip")?
            }
        }
    }

    Ok(Attr {
        description,
        format,
        r#type,
        as_schema,
        as_schema_generic,
        rename,
        required,
        min_items,
        max_items,
        nullable,
        skip,
    })
}

pub(super) fn find_attrs<const N: usize>(
    set_attrs: [&'static str; N],
    attrs: &[Attribute],
    owner: &str,
) -> Result<[Option<LitStr>; N], Error> {
    let mut sa = SetAttributes::<0>::var(set_attrs);
    sa.find_attrs(attrs, owner)?;

    let mut out = [const { None }; N];

    for (i, set_attr) in set_attrs.iter().enumerate() {
        out[i] = sa.extract_literal(set_attr).map(Into::into)
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
enum ArgTaking {
    Takes,
    MayNot,
    #[allow(unused)]
    MustNot,
}

impl Default for ArgTaking {
    fn default() -> Self {
        Self::Takes
    }
}

enum Value {
    LitStr(syn::LitStr),
    Empty,
}

#[derive(Default)]
struct AttrProp {
    value: Option<Value>,
    arg_taking: ArgTaking,
    takes_one_of: Vec<&'static str>,
}

impl AttrProp {
    fn one_of_for_error(&self) -> String {
        format_possible_values(self.takes_one_of.iter().collect(), "or")
    }
}

struct SetAttributes<const K: usize>(HashMap<&'static str, AttrProp>, Option<[&'static str; K]>);

impl<const K: usize> SetAttributes<K> {
    fn var<const N: usize>(attrs: [&'static str; N]) -> Self {
        let mut m = HashMap::with_capacity(attrs.len());
        for attr in attrs {
            m.insert(attr, Default::default());
        }
        Self(m, None)
    }

    // re_var clears all former variables and disallows
    fn re_var<const N: usize>(mut self, attrs: [&'static str; N]) -> Self {
        self.clear();
        self.1 = None;

        for attr in attrs {
            self.insert(attr, Default::default());
        }
        self
    }

    fn disallow(mut self, disallow: Option<[&'static str; K]>) -> Self {
        if let Some(disallow) = disallow {
            for attr in disallow {
                _ = self.remove(attr);
            }
        }

        self.1 = disallow;
        self
    }

    fn is_disallowed(&self, attr: &str) -> bool {
        if let Some(ref d) = self.1 {
            d.contains(&attr)
        } else {
            false
        }
    }

    #[allow(unused)]
    fn disallow_argument<const N: usize>(mut self, attrs: [&'static str; N]) -> Self {
        for attr in attrs {
            self.get_mut(&attr)
                .unwrap_or_else(|| panic!("{attr} should exist"))
                .arg_taking = ArgTaking::MustNot
        }

        self
    }

    fn allow_bool<const N: usize>(mut self, attrs: [&'static str; N]) -> Self {
        for attr in attrs {
            self = self.allow_one_of_(attr, ["true", "false"], true)
        }

        self
    }

    fn only_allow_one_of<const N: usize>(self, attr: &str, one_ofs: [&'static str; N]) -> Self {
        self.allow_one_of_(attr, one_ofs, false)
    }

    fn allow_one_of_<const N: usize>(
        mut self,
        attr: &str,
        one_ofs: [&'static str; N],
        maybe_empty: bool,
    ) -> Self {
        let a = self
            .get_mut(attr)
            .unwrap_or_else(|| panic!("{attr} should exist"));
        if maybe_empty {
            a.arg_taking = ArgTaking::MayNot;
        }
        a.takes_one_of = one_ofs.to_vec();

        self
    }

    fn extract_int(&mut self, name: &str) -> Result<Option<i64>, Error> {
        if let Some(v) = self.extract_literal(name) {
            v.value()
                .parse()
                .map_err(|err| Error::new(v.span(), format!("schema attribute {name}: {err}")))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn extract_bool(&mut self, name: &str) -> Result<Option<bool>, Error> {
        Ok(self.extract(name)?.map(|v| v.to_lowercase() != *"false"))
    }

    fn extract_literal(&mut self, name: &str) -> Option<syn::LitStr> {
        if let Some(attr_prop) = self.extract_attr_prop(name) {
            match attr_prop.value {
                Some(Value::LitStr(lit)) => Some(lit),
                None => None,
                _ => panic!("{name} is not syn::LitStr"),
            }
        } else {
            None
        }
    }

    fn extract(&mut self, name: &str) -> Result<Option<String>, Error> {
        if let Some(attr_prop) = self.extract_attr_prop(name) {
            match &attr_prop.value {
                Some(Value::LitStr(lit)) => {
                    let value = lit.value();
                    if attr_prop.takes_one_of.is_empty()
                        || attr_prop.takes_one_of.contains(&value.as_str())
                    {
                        Ok(Some(value))
                    } else {
                        Err(Error::new(
                            lit.span(),
                            format!(
                                "schema attribute {} only takes one of: {}",
                                name,
                                attr_prop.one_of_for_error()
                            ),
                        ))
                    }
                }
                Some(Value::Empty) => Ok(Some(String::new())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn extract_attr_prop(&mut self, name: &str) -> Option<AttrProp> {
        if self.1.is_some_and(|disallow| disallow.contains(&name)) {
            if self.contains_key(name) {
                panic!("unexpected attribute {name}");
            }
            return None;
        }

        Some(
            self.remove(name)
                .unwrap_or_else(|| panic!("{name} should exist")),
        )
    }

    fn find_attrs(&mut self, attrs: &[Attribute], owner: &str) -> Result<(), Error> {
        if let Err(err) = self.get_attrs_(attrs, owner, true) {
            // I don't understand either (see test unread_attribute)
            if err.to_string().contains("expected `,`") {
                Ok(())
            } else {
                Err(err)
            }
        } else {
            Ok(())
        }
    }

    fn get_attrs(&mut self, attrs: &[Attribute], owner: &str) -> Result<(), Error> {
        self.get_attrs_(attrs, owner, false)
    }

    fn get_attrs_(&mut self, attrs: &[Attribute], owner: &str, finding: bool) -> Result<(), Error> {
        for attr in attrs {
            if attr.path().is_ident(owner) {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        let s_attr = ident.to_string();
                        if let Some(attr_prop) = self.get_mut(s_attr.as_str()) {
                            match (meta.value(), attr_prop.arg_taking) {
                                (Ok(value), ArgTaking::Takes | ArgTaking::MayNot) => {
                                    attr_prop.value = Some(Value::LitStr(value.parse()?));
                                }
                                (Ok(_), ArgTaking::MustNot) => {
                                    return Err(meta.error(format!(
                                        "schema attribute {s_attr} takes no argument"
                                    )))
                                }
                                (Err(_), ArgTaking::Takes) => {
                                    return Err(meta.error(format!(
                                        "schema attribute {s_attr} needs argument"
                                    )))
                                }
                                (Err(_), ArgTaking::MayNot | ArgTaking::MustNot) => {
                                    attr_prop.value = Some(Value::Empty)
                                }
                            };
                        } else if !finding {
                            return Err(meta.error(format!(
                                "{} schema attribute {s_attr}. Valid attributes include: {}.",
                                if self.is_disallowed(s_attr.as_str()) {
                                    "disallowed"
                                } else {
                                    "unsupported"
                                },
                                self.attr_for_error()
                            )));
                        }
                    };
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    fn attr_for_error(&self) -> String {
        format_possible_values(self.keys().collect::<Vec<_>>(), "and")
    }
}

fn format_possible_values(mut ps: Vec<&&str>, and_or: &str) -> String {
    let mut out = String::new();
    ps.sort();

    let len = ps.len() as i64;
    for (i, p) in ps.iter().enumerate() {
        out.push('`');
        out.push_str(p);
        out.push('`');

        if i as i64 == len - 2 {
            out.push_str(", ");
            out.push_str(and_or);
            out.push(' ');
        } else if i as i64 != len - 1 {
            out.push_str(", ");
        }
    }
    out
}

impl<const K: usize> Deref for SetAttributes<K> {
    type Target = HashMap<&'static str, AttrProp>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const K: usize> DerefMut for SetAttributes<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use crate::attr::{parse_enum, parse_field, Attr};
    use syn::{parse_quote, Attribute, Data, DataStruct, Fields};

    #[test]
    fn unread_attribute() {
        let attrs = get_fields_attrs(parse_quote!(
            struct A {
                #[attr(I_care_less = "something")]
                field: String,
            }
        ));

        for attr in &attrs[0] {
            let result = if attr.path().is_ident("attr") {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        if ident == "what_I_want" {
                            unimplemented!();
                        }
                        // If we Err here, we won't get the error
                    };
                    Ok(())
                })
            } else {
                continue;
            };
            assert!(result.unwrap_err().to_string().contains("expected `,`"))
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

            for (ith, field_attrs) in fields_attrs.iter().enumerate() {
                match parse_field(field_attrs, false) {
                    Ok(attr) => assert_eq!(&attr, &test.want[ith]),
                    Err(err) => panic!("test failed: {err}"),
                };
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

pub(crate) use case::{rename_all, rename_all_variants};

mod case {
    pub(crate) static SUPPORTED: [&str; 8] = [
        "camelCase",
        "snake_case",
        "lowercase",
        "UPPERCASE",
        "PascalCase",
        "SCREAMING_SNAKE_CASE",
        "kebab-case",
        "SCREAMING-KEBAB-CASE",
    ];

    struct PascalCase;

    impl PascalCase {
        fn to_pascal_case(name: &str) -> String {
            name.to_owned()
        }

        fn to_camel_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for (i, part) in parts.iter().enumerate() {
                if i == 0 {
                    out.push_str(&part.to_ascii_lowercase());
                } else {
                    out.push_str(
                        &(part[..1].to_ascii_uppercase() + &part[1..].to_ascii_lowercase()),
                    );
                }
            }

            out
        }

        fn to_snake_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            if parts.is_empty() {
                return out;
            }

            let last = parts.len() - 1;

            for (i, part) in parts.iter().enumerate() {
                out.push_str(&part.to_ascii_lowercase());

                if i != last {
                    out.push('_');
                }
            }

            out
        }

        fn tokenize(name: &str) -> Vec<&str> {
            let mut out = Vec::new();
            #[derive(Clone, Copy)]
            #[allow(clippy::enum_variant_names)]
            enum Last {
                IsUpper,
                IsLower,
                IsPartFirst,
            }
            let mut last = Last::IsPartFirst;
            let mut cursor = 0;
            let mut current_part = 0;

            macro_rules! new_part {
                (0) => {{
                    out.push(&name[..]);
                    cursor = 0;
                }};
                ($i:tt, $cursor_incr:expr) => {{
                    let new_cursor = ($i as i64 + $cursor_incr) as usize;
                    out[current_part] = &name[cursor..new_cursor];
                    out.push(&name[new_cursor..]);
                    cursor = new_cursor;
                    current_part += 1;
                }};
            }

            for (i, c) in name.char_indices() {
                let is_upper = c.is_uppercase();
                match (i, is_upper, last) {
                    (0, _, _) => {
                        new_part!(0);
                        continue;
                    }
                    (_, true, Last::IsLower) => {
                        new_part!(i, 0);
                        last = Last::IsPartFirst;
                        continue;
                    }
                    (_, false, Last::IsUpper) => new_part!(i, -1),
                    _ => {}
                }

                if is_upper {
                    last = Last::IsUpper
                } else {
                    last = Last::IsLower
                }
            }

            out
        }

        fn to_kebab_case(name: &str) -> String {
            Self::to_snake_case(name).replace("_", "-")
        }
    }

    #[allow(non_camel_case_types)]
    struct snake_case;

    impl snake_case {
        fn to_snake_case(name: &str) -> String {
            name.to_owned()
        }

        fn to_pascal_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for part in parts {
                out.push_str(&(part[..1].to_ascii_uppercase() + &part[1..]));
            }

            out
        }

        fn to_camel_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for (i, part) in parts.iter().enumerate() {
                if i == 0 {
                    out.push_str(&(part[..1].to_ascii_lowercase() + &part[1..]));
                } else {
                    out.push_str(&(part[..1].to_ascii_uppercase() + &part[1..]));
                }
            }

            out
        }

        fn tokenize(name: &str) -> Vec<&str> {
            name.split('_').filter(|s| !s.is_empty()).collect()
        }

        fn to_kebab_case(name: &str) -> String {
            name.replace("_", "-")
        }
    }

    macro_rules! SCREAM {
        ($case:ident => { $($method:ident => $converter:ident),+ }) => {
            impl $case {
                $(
                    #[allow(non_snake_case)]
                    fn $method(name: &str) -> String {
                        $case::$converter(name).to_uppercase()
                    }
                )+
            }
        };
    }

    SCREAM!(snake_case => {
        SCREAMING_SNAKE_CASE => to_snake_case,
        SCREAMING_KEBAB_CASE => to_kebab_case
    });

    SCREAM!(PascalCase => {
        SCREAMING_SNAKE_CASE => to_snake_case,
        SCREAMING_KEBAB_CASE => to_kebab_case
    });

    macro_rules! rn_all {
        ($case:ident, $name:ident) => {
            #[allow(non_snake_case)]
            pub(crate) fn $name(style: &str) -> Option<fn(&str) -> String> {
                match style {
                    "camelCase" => Some($case::to_camel_case),
                    "snake_case" => Some($case::to_snake_case),
                    "lowercase" => Some(lowercase),
                    "UPPERCASE" => Some(UPPERCASE),
                    "PascalCase" => Some($case::to_pascal_case),
                    "SCREAMING_SNAKE_CASE" => Some($case::SCREAMING_SNAKE_CASE),
                    "kebab-case" => Some($case::to_kebab_case),
                    "SCREAMING-KEBAB-CASE" => Some($case::SCREAMING_KEBAB_CASE),
                    _ => None,
                }
            }
        };
    }

    rn_all!(snake_case, rename_all);
    rn_all!(PascalCase, rename_all_variants);

    fn lowercase(field_name: &str) -> String {
        field_name.to_ascii_lowercase()
    }

    #[allow(non_snake_case)]
    fn UPPERCASE(field_name: &str) -> String {
        field_name.to_ascii_uppercase()
    }

    #[cfg(test)]
    mod test {
        use crate::attr::case::{snake_case, PascalCase};
        #[allow(non_camel_case_types)]
        enum Case {
            camel,
            snake,
            Pascal,
        }
        #[test]
        fn from_snake() {
            struct Test {
                title: &'static str,
                input: &'static str,
                wants: Vec<(Case, &'static str)>,
            }

            let tests = [
                Test {
                    title: "leading delim",
                    input: "__private",
                    wants: vec![(Case::camel, "private"), (Case::Pascal, "Private")],
                },
                Test {
                    title: "normal snake_case",
                    input: "hello_world",
                    wants: vec![(Case::camel, "helloWorld"), (Case::Pascal, "HelloWorld")],
                },
                Test {
                    title: "`_` mayhem",
                    input: "__foo__Bar__",
                    wants: vec![(Case::camel, "fooBar"), (Case::Pascal, "FooBar")],
                },
                Test {
                    title: "alreadyCamel_alreadyCamel",
                    input: "alreadyCamel_alreadyCamel",
                    wants: vec![
                        (Case::camel, "alreadyCamelAlreadyCamel"),
                        (Case::Pascal, "AlreadyCamelAlreadyCamel"),
                    ],
                },
                Test {
                    title: "alreadyCamel",
                    input: "alreadyCamel",
                    wants: vec![
                        (Case::camel, "alreadyCamel"),
                        (Case::Pascal, "AlreadyCamel"),
                    ],
                },
            ];

            for test in tests {
                println!("{}", test.title);
                for want in test.wants {
                    match want {
                        (Case::camel, want) => {
                            assert_eq!(snake_case::to_camel_case(test.input), want)
                        }
                        (Case::snake, want) => {
                            assert_eq!(snake_case::to_snake_case(test.input), want)
                        }
                        (Case::Pascal, want) => {
                            assert_eq!(snake_case::to_pascal_case(test.input), want)
                        }
                    }
                }
            }
        }

        #[test]
        #[allow(non_snake_case)]
        fn PascalCaseTokenize() {
            let tests = [
                ("HTTPRequest", vec!["HTTP", "Request"]),
                ("LiFE", vec!["Li", "FE"]),
                ("PipE", vec!["Pip", "E"]),
                ("NormalPascal", vec!["Normal", "Pascal"]),
                ("invalidPascal", vec!["invalid", "Pascal"]),
                ("very_invalid_pascal", vec!["very_invalid_pascal"]),
            ];

            for test in tests {
                assert_eq!(PascalCase::tokenize(test.0), test.1)
            }
        }

        #[test]
        #[allow(non_snake_case)]
        fn from_PascalCase() {
            struct Test {
                title: &'static str,
                input: &'static str,
                wants: Vec<(Case, &'static str)>,
            }

            let tests = [
                Test {
                    title: "consecutive capitals",
                    input: "HTTPRequest",
                    wants: vec![(Case::snake, "http_request"), (Case::camel, "httpRequest")],
                },
                Test {
                    title: "consecutive capitals (1)",
                    input: "MyHTTPRequest",
                    wants: vec![(Case::snake, "my_http_request")],
                },
                Test {
                    title: "consecutive capitals (2)",
                    input: "ABCdef",
                    wants: vec![(Case::snake, "ab_cdef")],
                },
                Test {
                    title: "consecutive capitals (3)",
                    input: "HTTPRequestAPI",
                    wants: vec![
                        (Case::snake, "http_request_api"),
                        (Case::camel, "httpRequestApi"),
                    ],
                },
                Test {
                    title: "normal PascalCase",
                    input: "HelloWorld",
                    wants: vec![(Case::snake, "hello_world"), (Case::camel, "helloWorld")],
                },
            ];

            for test in tests {
                println!("{}", test.title);
                for want in test.wants {
                    match want {
                        (Case::camel, want) => {
                            assert_eq!(PascalCase::to_camel_case(test.input), want)
                        }
                        (Case::snake, want) => {
                            assert_eq!(PascalCase::to_snake_case(test.input), want)
                        }
                        (Case::Pascal, want) => {
                            assert_eq!(PascalCase::to_pascal_case(test.input), want)
                        }
                    }
                }
            }
        }
    }
}
