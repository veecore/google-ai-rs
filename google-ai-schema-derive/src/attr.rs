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
    fmt::{Debug, Display, Write as _},
    ops::Deref,
    str::FromStr,
};

use case::Case;
use proc_macro2::Span;
use syn::{meta::ParseNestedMeta, parse::Parse, Attribute, Error};

// see as a method on SetAttr
macro_rules! get_attrs {
    ($set:ident => {
        $(
            let $attr:ident $(as $attr_as:tt)? $(= $val:expr)?; // Type-inference can do most of the job
        )*
    }) => {
        $(
            // not storing $val will cause unnecessary re-computing
            //
            // wouldn't have been a tuple if we had paste to give the fn a new name
            let mut $attr = (None, get_attrs!(@unwrap_or $($val)?, new_attr()));
        )*

        for attr in $set.attrs {
            if attr.path().is_ident($set.owner) {
                attr.parse_nested_meta(|meta| {
                    if let Some(ident) = meta.path.get_ident() {
                        let s_attr = ident.to_string();
                        let s_attr = s_attr.as_str();

                        if !$set.is_finding && $set.is_disallowed(&s_attr) {
                            return Err(meta.error(format!(
                                "Disallowed schema attribute {s_attr}. Allowed attributes include: {}",
                                $set.attr_for_error(&mut [$(get_attrs!(@unwrap_or $($attr_as)?, stringify!($attr))),*])
                            )))
                        }

                        $(
                            if s_attr == get_attrs!(@unwrap_or $($attr_as)?, stringify!($attr)) {
                                $attr.0 = ($attr.1)($attr.0.take(), &meta).map_err(|err| {
                                    // FIXME
                                    // let mut prefix = meta.error(format!("Schema attribute {s_attr}: "));
                                    // prefix.combine(err);
                                    // prefix
                                    let msg = format!("Schema attribute {s_attr}: {err}");
                                    Error::new(err.span(), msg)
                                })?;
                                return Ok(())
                            }
                        )*

                        if !$set.is_finding {
                            Err(meta.error(format!(
                                "Unsupported schema attribute {s_attr}. Valid attributes include: {}",
                                $set.attr_for_error(&mut [$(get_attrs!(@unwrap_or $($attr_as)?, stringify!($attr))),*])
                            )))
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }).or_else(|err| {
                    // I don't understand either (see test unread_attribute)
                    if $set.is_finding && err.to_string().contains("expected `,`") {
                        Ok(())
                    } else {
                        Err(err)
                    }
                })?;
            }
        }

        $(
            let $attr = $attr.0;
        )*
    };
    (@unwrap_or $val:expr, $alt:expr) => {$val};
    (@unwrap_or , $alt:expr) => {$alt};

    {@attr_as $attr_as:tt or $var:ident} => {$attr_as};
    {@attr_as or $var:ident} => {stringify!($var)};
}

/// Top-level type attributes for schema generation
#[derive(Default)]
pub(crate) struct TopAttr {
    pub(crate) description: Option<String>,
    pub(crate) rename_all: Option<Case>,
    pub(crate) rename_all_with: Option<syn::ExprPath>,
    pub(crate) crate_path: Option<syn::Path>,
    pub(crate) nullable: Option<bool>,
    pub(crate) ignore_serde: Option<bool>,
}

pub(crate) fn parse_top(attrs: &[Attribute]) -> Result<TopAttr, Error> {
    let attrs = SetAttr::new(attrs);

    let rename_all_attr = new_attr::<syn::LitStr, Case>();
    get_attrs! {
        attrs => {
            let description = new_attr_string_concat();
            let rename_all = rename_all_attr;
            let rename_all_with = new_attr_expr_path();
            let crate_path = new_attr_path();
            let nullable = new_attr_bool();
            let ignore_serde = new_attr_bool();
        }
    }

    let mut any_rename_all = rename_all;

    if ignore_serde.is_none_or(|ignore_serde| !ignore_serde) && any_rename_all.is_none() {
        // let's use serde's rename
        let attrs = attrs.switch_to_serde();
        get_attrs! {
            attrs => {
                let rename_all = rename_all_attr;
            }
        }
        any_rename_all = rename_all;
    }

    Ok(TopAttr {
        description,
        rename_all: any_rename_all,
        rename_all_with,
        crate_path,
        nullable,
        ignore_serde,
    })
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

impl From<&str> for LitStr {
    fn from(value: &str) -> Self {
        Self(syn::LitStr::new(value, Span::call_site()))
    }
}

impl Deref for LitStr {
    type Target = syn::LitStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Attr {
    pub(crate) description: Option<String>,
    pub(crate) format: Option<Spanned<Format>>,
    pub(crate) r#type: Option<Spanned<Type>>,
    pub(crate) as_schema: Option<syn::ExprPath>,
    pub(crate) as_schema_generic: Option<syn::ExprPath>,
    pub(crate) rename: Option<String>,
    pub(crate) required: Option<bool>,
    pub(crate) min_items: Option<i64>,
    pub(crate) max_items: Option<i64>,
    pub(crate) nullable: Option<bool>,
    pub(crate) skip: Option<bool>,
}

pub(crate) fn parse_field(attrs: &[Attribute], ignore_serde: bool) -> Result<Attr, Error> {
    parse_item(attrs, ignore_serde, None)
}

pub(crate) fn parse_plain_enum(attrs: &[Attribute], ignore_serde: bool) -> Result<Attr, Error> {
    parse_item(
        attrs,
        ignore_serde,
        Some(&[
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
    parse_item(attrs, ignore_serde, Some(&["rename"]))
}

fn parse_item(
    attrs: &[Attribute],
    ignore_serde: bool,
    disallow: Option<&'static [&'static str]>,
) -> Result<Attr, Error> {
    let mut attrs = SetAttr::new(attrs);
    if let Some(disallow) = disallow {
        attrs = attrs.disallow(disallow);
    }

    let rename_attr = new_attr();
    let skip_attr = new_attr_bool();
    get_attrs! {
        attrs => {
            let description = new_attr_string_concat();
            let format;
            let r#type;
            let as_schema = new_attr_expr_path();
            let as_schema_generic = new_attr_expr_path();
            let rename = rename_attr;
            let required = new_attr_bool();
            let min_items;
            let max_items;
            let nullable = new_attr_bool();
            let skip = skip_attr;
        }
    }

    let mut any_rename = rename;
    let mut any_skip = skip;

    if !ignore_serde {
        let get_rename = any_rename.is_none() && !attrs.is_disallowed(&"rename");
        let get_skip = any_skip.is_none() && !attrs.is_disallowed(&"skip");

        if get_rename || get_skip {
            attrs = attrs.switch_to_serde();
            get_attrs! {
                attrs => {
                    let rename = rename_attr;
                    let skip = skip_attr;
                }
            };

            if get_rename {
                any_rename = rename;
            }

            if get_skip {
                any_skip = skip
            }
        }
    }

    Ok(Attr {
        description,
        format,
        r#type,
        as_schema,
        as_schema_generic,
        rename: any_rename,
        required,
        min_items,
        max_items,
        nullable,
        skip: any_skip,
    })
}

// Just TryFrom
pub trait TryFromParse<T>: Sized {
    fn try_from_parse(parse: T) -> Result<Self, Error>;

    // FIXME: This shouldn't be T depenedent.
    fn try_from_nothing() -> Result<Self, ()> {
        Err(())
    }
}

impl<T: Parse> TryFromParse<T> for T {
    fn try_from_parse(parse: T) -> Result<Self, Error> {
        Ok(parse)
    }
}

// FIXME: Wasteful
impl TryFromParse<syn::LitStr> for syn::ExprPath {
    fn try_from_parse(parse: syn::LitStr) -> Result<Self, Error> {
        parse.parse()
    }
}

// FIXME: Wasteful
impl TryFromParse<syn::LitStr> for syn::Path {
    fn try_from_parse(parse: syn::LitStr) -> Result<Self, Error> {
        parse.parse()
    }
}

impl TryFromParse<syn::LitInt> for i64 {
    fn try_from_parse(parse: syn::LitInt) -> Result<Self, Error> {
        parse.base10_parse()
    }
}

impl TryFromParse<syn::LitStr> for LitStr {
    fn try_from_parse(parse: syn::LitStr) -> Result<Self, Error> {
        Ok(LitStr(parse))
    }
}

impl TryFromParse<syn::LitStr> for String {
    fn try_from_parse(parse: syn::LitStr) -> Result<Self, Error> {
        Ok(parse.value())
    }
}

impl TryFromParse<syn::LitBool> for bool {
    fn try_from_parse(parse: syn::LitBool) -> Result<Self, Error> {
        Ok(parse.value)
    }

    fn try_from_nothing() -> Result<Self, ()> {
        // Presence is taken as truthiness
        Ok(true)
    }
}

impl TryFromParse<syn::LitStr> for bool {
    fn try_from_parse(parse: syn::LitStr) -> Result<Self, Error> {
        parse
            .value()
            .parse()
            .map_err(|_| Error::new(parse.span(), "Expected one of \"true\" or \"false\""))
    }

    fn try_from_nothing() -> Result<Self, ()> {
        // Presence is taken as truthiness
        Ok(true)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl<T> Display for Spanned<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Spanned<T> {
    pub fn error(&self, message: impl Display) -> Error {
        Error::new(self.span, message)
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    #[allow(dead_code)]
    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T> Spanned<T>
where
    T: Copy,
{
    pub fn value(&self) -> T {
        self.inner
    }
}

impl<T> PartialEq<Spanned<T>> for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Spanned<T>) -> bool {
        self.inner == other.inner
    }
}

impl<P, T> TryFromParse<P> for Spanned<T>
where
    P: Parse + syn::spanned::Spanned,
    T: TryFromParse<P>,
{
    fn try_from_parse(parse: P) -> Result<Self, Error> {
        Ok(Self {
            span: parse.span(),
            inner: T::try_from_parse(parse)?,
        })
    }

    fn try_from_nothing() -> Result<Self, ()> {
        T::try_from_nothing().map(|inner| Self {
            inner,
            span: Span::call_site(),
        })
    }
}

impl<T> FromStr for Spanned<T>
where
    T: FromStr,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: T::from_str(s)?,
            span: Span::call_site(),
        })
    }
}

fn new_attr<F, T>() -> impl Fn(Option<T>, &ParseNestedMeta<'_>) -> Result<Option<T>, Error> + Copy
where
    F: Parse,
    T: TryFromParse<F>,
{
    |former, new_value| {
        if former.is_some() {
            return Err(new_value.error("Multiple values not supported"));
        }
        match new_value.value() {
            Ok(v) => v
                .parse()
                .map_err(|err| new_value.error(format!("Error parsing value: {err}")))
                .map(|f| T::try_from_parse(f))
                .and_then(|r| r.map(Some)),
            Err(_) => T::try_from_nothing()
                .map(Some)
                .map_err(|_| new_value.error("Argument required")),
        }
    }
}

fn new_attr_string_concat(
) -> impl Fn(Option<String>, &ParseNestedMeta<'_>) -> Result<Option<String>, Error> {
    let base = new_attr::<syn::LitStr, String>();

    move |former, new_value| {
        base(None, new_value).map(|new| match (former, new) {
            (None, Some(new)) => Some(new),
            (Some(former), None) => Some(former),
            (Some(former), Some(new)) => {
                if new.is_empty() {
                    // Like doc
                    Some(format!("{former}\n{new}"))
                } else {
                    Some(format!("{former}{new}"))
                }
            },
            _ => None,
        })
    }
}

fn new_attr_any2<V: Parse, V1: Parse, T>(
) -> impl Fn(Option<T>, &ParseNestedMeta<'_>) -> Result<Option<T>, Error> + Copy
where
    T: Clone,
    T: TryFromParse<V>,
    T: TryFromParse<V1>,
{
    let from_v = new_attr::<V, T>();
    let from_v1 = new_attr::<V1, T>();

    move |former, new_value| {
        #[allow(dead_code)]
        struct ParseNestedMetaE<'a> {
            pub path: syn::Path,
            pub input: syn::parse::ParseStream<'a>,
        }

        let fork = new_value.input.fork();
        from_v(former.clone(), new_value).or_else(|_| {
            from_v1(former, unsafe {
                &std::mem::transmute::<ParseNestedMetaE, ParseNestedMeta>(ParseNestedMetaE {
                    path: new_value.path.clone(),
                    input: &fork,
                })
            })
        })
    }
}

fn new_attr_bool(
) -> impl Fn(Option<bool>, &ParseNestedMeta<'_>) -> Result<Option<bool>, Error> + Copy {
    new_attr_any2::<syn::LitStr, syn::LitBool, bool>()
}

// FIXME
fn new_attr_expr_path(
) -> impl Fn(Option<syn::ExprPath>, &ParseNestedMeta<'_>) -> Result<Option<syn::ExprPath>, Error> + Copy
{
    new_attr_any2::<syn::LitStr, syn::ExprPath, syn::ExprPath>()
}

// FIXME
pub(crate) fn new_attr_path(
) -> impl Fn(Option<syn::Path>, &ParseNestedMeta<'_>) -> Result<Option<syn::Path>, Error> + Copy {
    new_attr_any2::<syn::LitStr, syn::Path, syn::Path>()
}

#[derive(Debug)]
pub struct SetAttr<'a> {
    attrs: &'a [Attribute],
    // Disallowed attributes
    disallow: &'static [&'static str],
    owner: &'static str,
    is_finding: bool,
}

impl<'a> SetAttr<'a> {
    pub fn new(attrs: &'a [Attribute]) -> Self {
        Self {
            attrs,
            disallow: &[],
            owner: "schema",
            is_finding: false,
        }
    }

    fn disallow(mut self, disallow: &'static [&str]) -> Self {
        self.disallow = disallow;
        self
    }

    fn is_disallowed(&self, attr: &&str) -> bool {
        self.disallow.contains(attr)
    }

    fn owner(mut self, owner: &'static str) -> Self {
        self.owner = owner;
        self
    }

    fn finding(mut self) -> Self {
        self.is_finding = true;
        self
    }

    pub fn switch_to_serde(self) -> Self {
        self.owner("serde").finding()
    }

    // filter away the disallowed ones
    fn attr_for_error(&self, all: &mut [&str]) -> impl Display {
        // FIXME: move disallowed down so we can view allowed part while also sorting
        let mut allowed = all
            .iter()
            .filter(|v| !self.is_disallowed(v))
            .collect::<Vec<_>>();
        format_possible_values(&mut allowed, "and")
    }

    pub fn get_attr_fn<T>(
        &self,
        attr: &'static str,
        f: impl Fn(Option<T>, &ParseNestedMeta<'_>) -> Result<Option<T>, Error>,
    ) -> Result<Option<T>, Error> {
        get_attrs! {
            self => {
                let attr_var as attr = f;
            }
        }

        Ok(attr_var)
    }

    pub fn find_serde_crate(attrs: &'a [Attribute]) -> Result<Option<syn::Path>, Error> {
        const SERDE_PATH_ATTR: &str = "crate"; // right?

        Self::new(attrs)
            .switch_to_serde()
            .get_attr_fn(SERDE_PATH_ATTR, new_attr_path())
    }
}

pub(crate) fn format_possible_values<V>(ps: &mut [V], and_or: &str) -> String
where
    V: Display + Ord,
{
    let mut out = String::new();
    ps.sort();

    let len = ps.len();
    for (i, p) in ps.iter().enumerate() {
        write!(&mut out, "`{p}`").unwrap();

        // Only apply ',' if there's something ahead
        if i + 1 < len {
            write!(&mut out, ", ").unwrap();
        }

        // Now, if we're the penultimate...
        // must be below writing of ', ' so we get ', {and_or} {value}'
        if i + 2 == len {
            write!(&mut out, "{and_or} ").unwrap();
        }
    }
    out
}

pub(crate) fn unknown_one_of_error<P, T, V>(value: V, valid: &mut [P], target: T) -> impl Display
where
    P: Display + Ord,
    T: Display,
    V: Display,
{
    format!(
        "Unknown value {value} for {target}. Valid values include: {}",
        format_possible_values(valid, "and")
    )
}

#[cfg(test)]
mod test {
    use crate::attr::{parse_field, parse_plain_enum, Attr};
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
                        // If we Err or consume the value here, we won't get the error
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
                error_like: Some(vec!["boolean", "true", "false"]),
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
                error_like: Some(vec!["argument"]),
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
                error_like: Some(vec!["argument"]),
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
                parse_plain_enum(first_field_attrs, false)
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
                            // err.to_string only gives the first message
                            let err = err.into_compile_error().to_string().to_lowercase();

                            for like in error_like {
                                matches = matches || err.contains(like)
                            }
                            println!("{err}");
                            assert!(matches);
                        }
                    }
                }
            } else if let Err(err) = r {
                panic!("test failed: {err:#?}");
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
                        r#type: Some("String".parse().unwrap()),
                        ..Default::default()
                    },
                    Attr {
                        r#type: Some("Number".parse().unwrap()),
                        format: Some("float".parse().unwrap()),
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
            Test {
                title: "boolean",
                input: parse_quote! {struct S {
                    #[schema(skip)]
                    rgb: String,
                    #[schema(nullable = true)]
                    nn: I,
                    #[schema(skip = "false")]
                    ff: F
                }},
                want: vec![
                    Attr {
                        skip: Some(true),
                        ..Default::default()
                    },
                    Attr {
                        nullable: Some(true),
                        ..Default::default()
                    },
                    Attr {
                        skip: Some(false),
                        ..Default::default()
                    },
                ],
            },
            Test {
                title: "description",
                input: parse_quote! {struct S {
                    #[schema(description = "Line 1")]
                    #[schema(description = "")]
                    #[schema(description = "Line 2")]

                    rgb: String,
                }},
                want: vec![Attr {
                    description: Some("Line 1\nLine 2".to_string()),
                    ..Default::default()
                }],
            },
            Test {
                title: "description -(1)",
                input: parse_quote! {struct S {
                    #[schema(description = "Line 1 ")]
                    #[schema(description = "Line 2")]

                    rgb: String,
                }},
                want: vec![Attr {
                    description: Some("Line 1 Line 2".to_string()),
                    ..Default::default()
                }],
            },
            Test {
                title: "ExprPath - FIXME(tired)",
                input: parse_quote! {struct S {
                    #[schema(as_schema = "crate::module::function")]
                    rgb: String,
                    // FIXME:
                    // #[schema(as_schema_generic = crate::module::function)]
                    // rgb_: String,
                }},
                want: vec![
                    Attr {
                        as_schema: Some(parse_quote!(crate::module::function)),
                        ..Default::default()
                    }, /*, Attr {
                           as_schema_generic: Some(parse_quote!(crate::module::function)),
                           ..Default::default()
                       }*/
                ],
            },
        ];

        for test in tests {
            println!("title: {}", test.title);
            let fields_attrs = get_fields_attrs(test.input);
            assert_eq!(fields_attrs.len(), test.want.len());

            for (ith, field_attrs) in fields_attrs.iter().enumerate() {
                match parse_field(field_attrs, false) {
                    Ok(attr) => assert_eq!(&attr, &test.want[ith]),
                    Err(err) => panic!("test failed: {err:#?}"),
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

use crate::{schema::Type, Format};

mod case {
    macro_rules! declare_enum_attr {
        (
            $(#[$meta:meta])*
            $vis:vis enum $ty:ident = $ty_parallel:ident {
                $(
                    $(#[$v_meta:meta])*
                    $variant:ident = $val:literal
                ),*
            }
        ) => {
            $(#[$meta])*
            $vis enum $ty {
                $(
                    $(#[$v_meta])*
                    $variant
                ),*
            }

            impl $crate::attr::TryFromParse<syn::LitStr> for $ty {
                fn try_from_parse(parse: syn::LitStr) -> Result<Self, syn::Error> {
                    let value = parse.value();
                    let span = parse.span();
                    value.parse().map_err(|_| {
                        let err = $crate::attr::unknown_one_of_error(value, &mut [$($val),*], stringify!($ty_parallel));
                        syn::Error::new(span, err)
                    })
                }
            }

            impl std::str::FromStr for $ty {
                type Err = $crate::schema::UnknownVariant;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $(
                            $val => Ok(Self::$variant),
                        )*
                        _ => Err($crate::schema::UnknownVariant)
                    }
                }
            }

            impl std::fmt::Display for $ty {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(
                            Self::$variant => f.write_str($val),
                        )*
                    }
                }
            }
        }
    }

    declare_enum_attr! {
        #[derive(Copy, Clone, Debug)]
        pub enum Case = Case {
            Camel = "camelCase",
            Snake = "snake_case",
            Lower = "lowercase",
            Upper = "UPPERCASE",
            Pascal = "PascalCase",
            ScreamingSnake = "SCREAMING_SNAKE_CASE",
            Kebab = "kebab-case",
            ScreamingKebab = "SCREAMING-KEBAB-CASE"
        }
    }

    // This is to avoid heap-alloc in to_ascii_* and for convenience
    trait StrExt {
        fn to_ascii_lowercase_iter(self) -> impl Iterator<Item = char>;
        fn to_ascii_uppercase_iter(self) -> impl Iterator<Item = char>;
    }

    impl StrExt for &str {
        fn to_ascii_lowercase_iter(self) -> impl Iterator<Item = char> {
            self.chars().map(|c| c.to_ascii_lowercase())
        }

        fn to_ascii_uppercase_iter(self) -> impl Iterator<Item = char> {
            self.chars().map(|c| c.to_ascii_uppercase())
        }
    }

    struct PascalCase;

    impl PascalCase {
        fn to_pascal_case(name: &str) -> String {
            name.to_owned()
        }

        fn to_camel_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for (i, part) in parts.enumerate() {
                if i == 0 {
                    out.extend(part.to_ascii_lowercase_iter());
                } else {
                    out.extend(part[..1].to_ascii_uppercase_iter());
                    out.extend(part[1..].to_ascii_lowercase_iter());
                }
            }

            out
        }

        fn to_snake_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for (i, part) in parts.enumerate() {
                if i != 0 {
                    out.push('_');
                }
                out.extend(part.to_ascii_lowercase_iter());
            }

            out
        }

        fn tokenize(name: &str) -> impl Iterator<Item = &str> {
            struct Parts<'a> {
                residue: &'a str,
            }

            impl<'a> Iterator for Parts<'a> {
                type Item = &'a str;

                fn next(&mut self) -> Option<Self::Item> {
                    // We're always at the start of a new part
                    let mut last_is_upper = true;

                    let mut cursor = self.residue.len();
                    for (i, c) in self.residue.char_indices() {
                        let is_upper = c.is_uppercase();

                        match (last_is_upper, is_upper) {
                            // aA
                            (false, true) => {
                                cursor = i;
                                break;
                            }
                            // AAa
                            (true, false) => {
                                // If there's something before last that must've been upper...
                                // If it weren't, it'd have been popped in the branch above on getting
                                // to last. So we check if we have at-least something before the last.
                                if i > 1 {
                                    cursor = i - 1;
                                    break;
                                }
                            }
                            // AA or aa
                            _ => {}
                        }

                        last_is_upper = is_upper;
                    }

                    // Must be above the overwrite
                    let next = &self.residue[..cursor];
                    self.residue = &self.residue[cursor..];

                    if next.is_empty() {
                        None
                    } else {
                        Some(next)
                    }
                }
            }

            Parts { residue: name }
        }

        fn to_kebab_case(name: &str) -> String {
            let mut out = Self::to_snake_case(name);

            unsafe {
                out.as_mut_vec().iter_mut().for_each(|c| {
                    if *c == b'_' {
                        *c = b'-'
                    }
                })
            };

            out
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
                out.extend(part[..1].to_ascii_uppercase_iter());
                out.push_str(&part[1..]);
            }

            out
        }

        fn to_camel_case(name: &str) -> String {
            let parts = Self::tokenize(name);
            let mut out = String::new();

            for (i, part) in parts.enumerate() {
                if i == 0 {
                    out.extend(part[..1].to_ascii_lowercase_iter());
                } else {
                    out.extend(part[..1].to_ascii_uppercase_iter());
                }
                out.push_str(&part[1..]);
            }

            out
        }

        fn tokenize(name: &str) -> impl Iterator<Item = &str> {
            name.split('_').filter(|s| !s.is_empty())
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
            pub(crate) fn $name(style: Case) -> fn(&str) -> String {
                match style {
                    Case::Camel => $case::to_camel_case,
                    Case::Snake => $case::to_snake_case,
                    Case::Lower => lowercase,
                    Case::Upper => UPPERCASE,
                    Case::Pascal => $case::to_pascal_case,
                    Case::ScreamingSnake => $case::SCREAMING_SNAKE_CASE,
                    Case::Kebab => $case::to_kebab_case,
                    Case::ScreamingKebab => $case::SCREAMING_KEBAB_CASE,
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
        use crate::attr::case::{snake_case, Case, PascalCase};
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
                    wants: vec![(Case::Camel, "private"), (Case::Pascal, "Private")],
                },
                Test {
                    title: "normal snake_case",
                    input: "hello_world",
                    wants: vec![(Case::Camel, "helloWorld"), (Case::Pascal, "HelloWorld")],
                },
                Test {
                    title: "`_` mayhem",
                    input: "__foo__Bar__",
                    wants: vec![(Case::Camel, "fooBar"), (Case::Pascal, "FooBar")],
                },
                Test {
                    title: "alreadyCamel_alreadyCamel",
                    input: "alreadyCamel_alreadyCamel",
                    wants: vec![
                        (Case::Camel, "alreadyCamelAlreadyCamel"),
                        (Case::Pascal, "AlreadyCamelAlreadyCamel"),
                    ],
                },
                Test {
                    title: "alreadyCamel",
                    input: "alreadyCamel",
                    wants: vec![
                        (Case::Camel, "alreadyCamel"),
                        (Case::Pascal, "AlreadyCamel"),
                    ],
                },
            ];

            for test in tests {
                println!("{}", test.title);
                for want in test.wants {
                    match want {
                        (Case::Camel, want) => {
                            assert_eq!(snake_case::to_camel_case(test.input), want)
                        }
                        (Case::Snake, want) => {
                            assert_eq!(snake_case::to_snake_case(test.input), want)
                        }
                        (Case::Pascal, want) => {
                            assert_eq!(snake_case::to_pascal_case(test.input), want)
                        }
                        _ => unimplemented!(),
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
                (
                    "NormalPascalLongerAJiGsAwTT",
                    vec!["Normal", "Pascal", "Longer", "A", "Ji", "Gs", "Aw", "TT"],
                ),
            ];

            for test in tests {
                assert_eq!(
                    PascalCase::tokenize(test.0).into_iter().collect::<Vec<_>>(),
                    test.1
                )
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
                    wants: vec![(Case::Snake, "http_request"), (Case::Camel, "httpRequest")],
                },
                Test {
                    title: "consecutive capitals (1)",
                    input: "MyHTTPRequest",
                    wants: vec![(Case::Snake, "my_http_request")],
                },
                Test {
                    title: "consecutive capitals (2)",
                    input: "ABCdef",
                    wants: vec![(Case::Snake, "ab_cdef")],
                },
                Test {
                    title: "consecutive capitals (3)",
                    input: "HTTPRequestAPI",
                    wants: vec![
                        (Case::Snake, "http_request_api"),
                        (Case::Camel, "httpRequestApi"),
                    ],
                },
                Test {
                    title: "normal PascalCase",
                    input: "HelloWorld",
                    wants: vec![
                        (Case::Snake, "hello_world"),
                        (Case::Camel, "helloWorld"),
                        (Case::Kebab, "hello-world"),
                    ],
                },
            ];

            for test in tests {
                println!("{}", test.title);
                for want in test.wants {
                    match want {
                        (Case::Camel, want) => {
                            assert_eq!(PascalCase::to_camel_case(test.input), want)
                        }
                        (Case::Snake, want) => {
                            assert_eq!(PascalCase::to_snake_case(test.input), want)
                        }
                        (Case::Pascal, want) => {
                            assert_eq!(PascalCase::to_pascal_case(test.input), want)
                        }
                        (Case::Kebab, want) => {
                            assert_eq!(PascalCase::to_kebab_case(test.input), want)
                        }
                        _ => todo!(),
                    }
                }
            }
        }
    }
}
