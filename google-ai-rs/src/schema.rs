use google_ai_schema_derive::AsSchema;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell, RefMut},
    collections::{BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    ffi::{CStr, CString},
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    rc::{Rc, Weak},
    sync::{
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
            AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        Arc, Mutex, RwLock, Weak as ArcWeak,
    },
};
use tokio::sync::{Mutex as TMutex, RwLock as TRwLock};

use crate::proto::{Schema, Type};

// SchemaType contains the list of OpenAPI data types as defined by
// https://spec.openapis.org/oas/v3.0.3#data-types
pub type SchemaType = Type;

/// A list of supported OpenAPI data formats for primitive types.
///
/// This enum provides a type-safe way to specify the `format` field of a `Schema`,
/// which is used to provide more detail about primitive data types.
#[derive(Clone, Copy, Debug)]
pub enum SchemaFormat {
    /// 32-bit floating-point number.
    Float,
    /// 64-bit floating-point number.
    Double,
    /// 32-bit integer.
    Int32,
    /// 64-bit integer.
    Int64,
    /// A string that can only be one of a predefined set of values.
    Enum,
    None,
}

impl SchemaFormat {
    /// Returns the string representation of the format.
    fn as_str(self) -> &'static str {
        match self {
            Self::Float => "float",
            Self::Double => "double",
            Self::Int32 => "int32",
            Self::Int64 => "int64",
            Self::Enum => "enum",
            Self::None => "",
        }
    }
}

impl Schema {
    /// Constructs a new schema for the specified primitive type.
    pub fn new(typ: SchemaType) -> Self {
        Schema {
            r#type: typ as i32,
            ..Default::default()
        }
    }

    /// Creates a new schema for an object type.
    pub fn new_object() -> Self {
        Self::new(Type::Object)
    }

    /// Creates a new schema for an array type.
    pub fn new_array() -> Self {
        Self::new(Type::Array)
    }

    /// Creates a new schema for a number type.
    pub fn new_number() -> Self {
        Self::new(Type::Number)
    }

    /// Creates a new schema for an integer type.
    pub fn new_integer() -> Self {
        Self::new(Type::Integer)
    }

    /// Creates a new schema for a string type.
    pub fn new_string() -> Self {
        Self::new(Type::String)
    }

    /// Sets the format of the schema.
    ///
    /// This is used for primitive types like `int32`, `int64`, `float`, `double`, or `enum`.
    pub fn format(mut self, format: SchemaFormat) -> Self {
        self.format = format.as_str().to_owned();
        self
    }

    /// Sets the description of the schema.
    ///
    /// The description can be formatted as Markdown.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets whether the schema value may be null.
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Sets the possible enum values for a `String` type.
    ///
    /// This method automatically sets the schema's `type` to `String` and its `format` to `Enum`.
    /// The values provided will be added to the `enum` field.
    ///
    /// # Example
    /// ```rust
    /// # use google_ai_rs::Schema;
    /// let enum_schema = Schema::new_string()
    ///     .into_enum(["EAST", "NORTH", "SOUTH", "WEST"]);
    /// ```
    pub fn into_enum<I, S>(self, r#enum: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        // We ensure the schema is a String type before applying enum properties.
        if self.is_string() {
            let mut self_with_format = self.format(SchemaFormat::Enum);
            self_with_format.r#enum = r#enum.into_iter().map(Into::into).collect();
            self_with_format
        } else {
            self
        }
    }

    /// Sets the schema for the elements of an `Array` type.
    ///
    /// This method is only effective when the schema's type is `Array`. It specifies the
    /// structure of the items contained within the array.
    ///
    /// # Example
    /// For a `Vec<String>`, you would define the schema like this:
    /// ```rust
    /// # use google_ai_rs::Schema;
    /// let string_array_schema = Schema::new_array()
    ///     .items(Schema::new_string());
    /// ```
    pub fn items(mut self, items: Schema) -> Self {
        if self.is_array() {
            self.items = Some(Box::new(items));
        }
        self
    }

    /// Sets the maximum number of elements for an `Array` schema.
    ///
    /// This method is only effective when the schema's type is `Array`.
    pub fn max_items(mut self, max_items: i64) -> Self {
        if self.is_array() {
            self.max_items = max_items;
        }
        self
    }

    /// Sets the minimum number of elements for an `Array` schema.
    ///
    /// This method is only effective when the schema's type is `Array`.
    pub fn min_items(mut self, min_items: i64) -> Self {
        if self.is_array() {
            self.min_items = min_items;
        }
        self
    }

    /// Adds a single property to an `Object` schema.
    ///
    /// This method is a convenience for adding a single key-value pair to the properties map.
    /// It's only effective when the schema's type is `Object`.
    ///
    /// # Arguments
    /// * `name` - The name of the property.
    /// * `schema` - The schema definition for the property.
    pub fn property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        if self.is_object() {
            self.properties.insert(name.into(), schema);
        }
        self
    }

    /// Sets the properties for an `Object` schema.
    ///
    /// This method is only effective when the schema's type is `Object`.
    ///
    /// # Arguments
    /// * `properties` - An iterator of key-value pairs where the key is the property
    ///   name and the value is the property's `Schema`.
    pub fn properties<I, S>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (S, Schema)>,
        S: Into<String>,
    {
        if self.is_object() {
            self.properties = properties.into_iter().map(|(k, v)| (k.into(), v)).collect();
        }
        self
    }

    /// Adds a required field to an `Object` schema.
    ///
    /// This method is only effective when the schema's type is `Object`.
    ///
    /// # Arguments
    /// * `name` - The name of the property that is now required.
    pub fn required_field(mut self, name: impl Into<String>) -> Self {
        if self.is_object() {
            self.required.push(name.into());
        }
        self
    }

    /// Sets the list of all required properties for an `Object` schema.
    ///
    /// This method is only effective when the schema's type is `Object`.
    ///
    /// # Arguments
    /// * `required` - An iterator of property names that must be present.
    pub fn required<I, S>(mut self, required: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        if self.is_object() {
            self.required = required.into_iter().map(Into::into).collect();
        }
        self
    }

    fn is_object(&self) -> bool {
        SchemaType::Object as i32 == self.r#type
    }

    fn is_array(&self) -> bool {
        SchemaType::Array as i32 == self.r#type
    }

    fn is_string(&self) -> bool {
        SchemaType::Object as i32 == self.r#type
    }
}

/// Trait for Rust types that can generate a `Schema` (a subset of OpenAPI schemas) automatically.
///
/// Implement this trait or derive `AsSchema` to enable schema generation for your types.
/// The derive macro supports extensive customization through attributes and integrates with Serde.
///
/// # Description Attributes
/// Descriptions can now span multiple lines. You can use multiple `#[schema(description = "...")]`
/// attributes on a field or a struct. Each attribute's content will be concatenated.
///
/// To add a new line, use an empty `#[schema(description = "")]` attribute, similar to how
/// the standard `///` documentation comments work.
///
/// # Examples
///
/// ```rust
/// use google_ai_rs::AsSchema;
///
/// #[derive(AsSchema)]
/// #[schema(rename_all = "camelCase")]
/// struct AiReport {
///     // A single-line description
///     #[schema(description = "This should include user's name and date of birth", required)]
///     data: String,
///
///     // A multi-line description
///     #[schema(description = "This field contains important metadata.")]
///     #[schema(description = "")] // New line
///     #[schema(description = "For example, the creation timestamp and author.")]
///     metadata: String,
/// }
/// ```
///
/// # Customizing Foreign Types
///
/// For types from other crates where you can't add the `derive`, you can either manually
/// specify its schema or reference a function that generates it.
///
/// **1. Using `r#type` and `format`:**
///
/// ```rust
/// use google_ai_rs::AsSchema;
/// # mod some_crate { pub struct TheirType; }
///
/// #[derive(AsSchema)]
/// struct AiReport {
///     #[schema(r#type = "Number", format = "double")]
///     foreign: some_crate::TheirType
/// }
/// ```
///
/// **2. Using `as_schema` with a function:**
///
/// This is useful for more complex foreign types.
///
/// ```rust
/// use google_ai_rs::{AsSchema, Schema, schema::SchemaFormat};
/// # mod some_crate { pub struct TheirType; }
///
/// #[derive(AsSchema)]
/// struct AiReport {
///     #[schema(as_schema = "foreign_type_schema")]
///     foreign: some_crate::TheirType
/// }
///
/// fn foreign_type_schema() -> Schema {
///     Schema::new_object()
///         .description("A custom schema for a foreign type.")
///         .property("id", Schema::new_number().format(SchemaFormat::Int64))
/// }
/// ```
///
/// # Serde Compatibility
///
/// The `AsSchema` derive macro automatically integrates with `serde` attributes for convenience.
///
/// - `#[serde(rename)]`/`#[serde(rename_all)]` are respected for naming fields in the schema.
/// - `#[serde(skip)]` fields are automatically excluded from the generated schema.
/// - You can disable Serde integration for a specific field with `#[schema(ignore_serde)]` or for the whole
///   type with `#[schema(serde_ignore)]` on the struct.
///
/// # Examples with Serde
///
/// ```rust
/// use google_ai_rs::AsSchema;
///
/// #[derive(AsSchema, serde::Deserialize)]
/// struct AiReport {
///     #[schema(description = "Important field", required)]
///     #[serde(rename = "json_field")] // Applies to the schema too
///     field: String,
///
///     #[serde(skip)] // Excluded from schema
///     internal: String,
///
///     #[schema(skip)] // Overrides Serde's behavior and excludes from schema
///     #[serde(rename = "count")] // This rename is ignored by the schema
///     item_count: i32,
/// }
/// ```
#[cfg_attr(
    not(no_diagnostic_namespace),
    diagnostic::on_unimplemented(
        note = "for local types consider adding `#[derive(google_ai_rs::AsSchema)]` to your `{Self}` type",
        note = "for types from other crates consider the as_schema attribute or check if you can represent with the r#type and format attributes",
        note = "consider google_ai_rs::Map for maps, and google_ai_rs::Tuple or derive AsSchemaWithSerde for tuples"
    )
)]
pub trait AsSchema {
    /// Generates the OpenAPI schema for this type
    fn as_schema() -> Schema;
}

impl<T: AsSchema + ?Sized> AsSchema for &T {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

impl<T: AsSchema + ?Sized> AsSchema for &mut T {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

impl<T: AsSchema + ?Sized> AsSchema for *const T {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

impl<T: AsSchema + ?Sized> AsSchema for *mut T {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

macro_rules! wrapper_generic {
    (
        $($ty:ident <$($life:lifetime, )* T $(: $b0:ident $(+ $b:ident)*)* $(, $g:ident : $gb:ident)*>)*
    ) => {
    	$(
	        impl<$($life,)* T $(, $g)*> AsSchema for $ty<$($life,)* T $(, $g)*>
	        where
	        	T: AsSchema $(+ $b0 $(+ $b)*)* + ?Sized,
                $($g: $gb,)*
	        {
	            fn as_schema() -> Schema {
	                T::as_schema()
	            }
	        }
        )*
    };
}

wrapper_generic! {
    Box<T>
    Arc<T>
    Rc<T>
    Mutex<T>
    RwLock<T>
    TMutex<T>
    TRwLock<T>
    Weak<T>
    ArcWeak<T>
    Cell<T>
    RefCell<T>
    PhantomData<T>
    RefMut<'a, T>
}

impl<'a, T: AsSchema + ToOwned + ?Sized + 'a> AsSchema for Cow<'a, T> {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

macro_rules! number {
    ($($n:ident, $ty:ident, $format:ident)*) => {
        $(impl AsSchema for $n {
            fn as_schema() -> Schema {
                Schema {
		            r#type: SchemaType::$ty as i32,
		            format: SchemaFormat::$format.as_str().into(),
		            ..Default::default()
		        }
            }
        })*
    };
}

number! {
    usize, Number, None
    u8, Number, None
    u16, Number, None
    u32, Number, None
    u64, Number, None
    u128, Number, None
    AtomicUsize, Number, None
    AtomicU8, Number, None
    AtomicU16, Number, None
    AtomicU32, Number, None
    AtomicU64, Number, None
    NonZeroUsize, Number, None
    NonZeroU8, Number, None
    NonZeroU16, Number, None
    NonZeroU32, Number, None
    NonZeroU64, Number, None
    NonZeroU128, Number, None
}

number! {
    isize, Integer, None
    i8, Integer, None
    i16, Integer, None
    i32, Integer, Int32
    i64, Integer, Int64
    i128, Integer, None
    AtomicIsize, Integer, None
    AtomicI8, Integer, None
    AtomicI16, Integer, None
    AtomicI32, Integer, Int32
    AtomicI64, Integer, Int64
    NonZeroIsize, Integer, None
    NonZeroI8, Integer, None
    NonZeroI16, Integer, None
    NonZeroI32, Integer, Int32
    NonZeroI64, Integer, Int64
    NonZeroI128, Integer, None
}

number! {
    f32, Number, Float
    f64, Number, Double
}

macro_rules! string {
    ($($n:ident)*) => {
    	$(
        impl AsSchema for $n {
            fn as_schema() -> Schema {
                Schema {
                    r#type: SchemaType::String as i32,
                    ..Default::default()
                }
            }
        })*
    };
}

string! {
    str
    String
    Path
    PathBuf
    char //
}

impl AsSchema for bool {
    fn as_schema() -> Schema {
        Schema {
            r#type: SchemaType::Boolean as i32,
            ..Default::default()
        }
    }
}

impl AsSchema for AtomicBool {
    fn as_schema() -> Schema {
        bool::as_schema()
    }
}

macro_rules! list_generic {
    (
        $($ty:ident <T $(: $b0:ident $(+ $b:ident)*)* $(, $g:ident : $gb:ident)*>)*
    ) => {
    	$(
	        impl<T $(, $g)*> AsSchema for $ty<T $(, $g)*>
	        where
	        	T: AsSchema $(+ $b0 $(+ $b)*)*,
	        	$($g: $gb,)*
	        {
	            fn as_schema() -> Schema {
	                Schema {
			            r#type: SchemaType::Array as i32,
			            items: Some(Box::new(T::as_schema())),
                        nullable: true,
			            ..Default::default()
			        }
	            }
	        }
        )*
    };
}

list_generic! {
    LinkedList<T>
    Vec<T>
    VecDeque<T>
    HashSet<T>
    BTreeSet<T>
    BinaryHeap<T>
}

impl AsSchema for CStr {
    fn as_schema() -> Schema {
        Vec::<u8>::as_schema()
    }
}

impl AsSchema for CString {
    fn as_schema() -> Schema {
        Vec::<u8>::as_schema()
    }
}

impl<T: AsSchema, const N: usize> AsSchema for [T; N] {
    fn as_schema() -> Schema {
        Schema {
            r#type: SchemaType::Array as i32,
            nullable: true,
            items: Some(Box::new(T::as_schema())),
            max_items: N as i64,
            min_items: N as i64,
            ..Default::default()
        }
    }
}

impl AsSchema for () {
    fn as_schema() -> Schema {
        Schema {
            r#type: SchemaType::Array as i32,
            nullable: true,
            ..Default::default()
        }
    }
}

impl<T: AsSchema> AsSchema for Option<T> {
    fn as_schema() -> Schema {
        let mut schema = T::as_schema();
        schema.nullable = true;
        schema
    }
}

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

macro_rules! custom_wrapper_utils {
    ($($name:ident)*) => {
        $(impl<T> Debug for $name<T>
        where
            T: Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                T::fmt(&self.inner, f)
            }
        }

        impl<T> Deref for $name<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<T> DerefMut for $name<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }

        impl<T> From<T> for $name<T> {
            fn from(value: T) -> Self {
                Self::new(value)
            }
        }

        impl<T> IntoIterator for $name<T>
        where
            T: IntoIterator
        {
            type Item = T::Item;
            type IntoIter = T::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T>
        where
            &'a T: IntoIterator
        {
            type Item = <&'a T as IntoIterator>::Item;
            type IntoIter = <&'a T as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }


        impl<'a, T> IntoIterator for &'a mut $name<T>
        where
            &'a mut T: IntoIterator
        {
            type Item = <&'a mut T as IntoIterator>::Item;
            type IntoIter = <&'a mut T as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl<T> $name<T> {
            pub fn new(inner: T) -> Self {
                Self { inner }
            }

            pub fn into_inner(self) -> T {
                self.inner
            }
        })*
    };
}

custom_wrapper_utils! {
    Tuple
    Map
}

/// A wrapper type to represent maps in Google Schema-friendly format.
///
/// Google Schema doesn't natively support maps, so this represents them as arrays
/// of key-value pairs using an `Entry` struct. Provides schema generation through
/// `AsSchema` and optional serde deserialization.
///
/// # Examples
///
/// Basic usage with HashMap:
/// ```
/// use std::collections::HashMap;
/// use google_ai_rs::{Map, Schema, AsSchema};
///
/// type MyMap = Map<HashMap<String, i32>>;
///
/// # use google_ai_rs::SchemaType;
/// let schema = Schema {
///     r#type: SchemaType::Array as i32,
///     items: Some(
///         Schema {
///             r#type: SchemaType::Object as i32,
///             properties: [
///                 ("key".to_string(), String::as_schema()),
///                 ("value".to_string(), i32::as_schema()),
///             ]
///             .into(),
///             required: ["key".to_string(), "value".to_string()].into(),
///             ..Default::default()
///         }
///         .into(),
///     ),
///     nullable: true,
///     ..Default::default()
/// };
///
/// assert_eq!(schema, MyMap::as_schema())
/// ```
///
/// Custom field identifiers and description:
/// ```
/// use google_ai_rs::{MapTrait, Map};
///
/// struct CustomMap;
/// impl MapTrait for CustomMap {
///     type Key = String;
///     type Value = i32;
///     const KEY_IDENT: &str = "id";
///     const VALUE_IDENT: &str = "count";
///     const DESCRIPTION: Option<&str> = Some("Custom mapped values");
/// }
///
/// type SpecialMap = Map<CustomMap>;
/// // Schema will have "id" and "count" fields with description
///
/// ```
/// **Deserialization Note:**  
/// Requires `serde` feature. Works best when `T` uses `MapAccess::next_entry` variants.
#[derive(Default)]
pub struct Map<T: ?Sized> {
    inner: T,
}

impl<T> AsSchema for Map<T>
where
    T: MapTrait,
    T::Key: AsSchema,
    T::Value: AsSchema,
{
    fn as_schema() -> Schema {
        let mut schema = Vec::<Entry<T>>::as_schema();
        if let Some(description) = T::DESCRIPTION {
            schema.description = description.to_owned()
        }
        schema
    }
}

/// Trait defining contract for types that can be represented as maps
///
/// # Examples
///
/// Implementing for a custom collection:
/// ```
/// use google_ai_rs::MapTrait;
///
/// struct PairList<K, V>(Vec<(K, V)>);
///
/// impl<K, V> MapTrait for PairList<K, V> {
///     type Key = K;
///     type Value = V;
///     const KEY_IDENT: &str = "k";
///     const VALUE_IDENT: &str = "v";
/// }
/// ```
pub trait MapTrait {
    type Key;
    type Value;
    const KEY_IDENT: &str = "key";
    const VALUE_IDENT: &str = "value";
    const DESCRIPTION: Option<&str> = None;
}

impl<K, V> MapTrait for HashMap<K, V> {
    type Key = K;

    type Value = V;
}

/// Internal representation of a map entry for schema generation
///
/// Automatically renames fields based on the MapTrait implementation.
///
/// # Example
///
/// With custom field identifiers:
/// ```
/// # use google_ai_rs::MapTrait;
/// struct Custom;
/// impl MapTrait for Custom {
///     type Key = String;
///     type Value = i32;
///     const KEY_IDENT: &str = "name";
///     const VALUE_IDENT: &str = "score";
/// }
///
/// // Entry<Custom> would generate schema fields "name" and "score"
/// ```
#[allow(dead_code)]
#[derive(AsSchema)]
#[schema(crate_path = "crate", rename_all_with = "Self::rename_idents")]
struct Entry<T>
where
    T: MapTrait,
{
    pub(super) key: T::Key,
    pub(super) value: T::Value,
}

impl<T> Entry<T>
where
    T: MapTrait,
{
    fn rename_idents(f: &str) -> String {
        match f {
            "key" => T::KEY_IDENT.to_owned(),
            "value" => T::VALUE_IDENT.to_owned(),
            _ => panic!("{f}"),
        }
    }
}

/// Wrapper type for tuple representation in Google Schema
///
/// Represents tuples as objects with positional field names ("0", "1", etc).
/// Supports tuples up to 16 elements.
///
/// # Example
///
/// ```
/// use google_ai_rs::{Tuple, Schema, AsSchema};
///
/// type StringIntPair = Tuple<(String, i32)>;
///
/// # use google_ai_rs::SchemaType;
/// let schema = Schema {
///     r#type: SchemaType::Object as i32,
///     properties: [
///         ("0".to_string(), String::as_schema()),
///         ("1".to_string(), i32::as_schema()),
///     ]
///     .into(),
///     required: ["0".to_string(), "1".to_string()].into(),
///     ..Default::default()
/// };
///
/// assert_eq!(schema, StringIntPair::as_schema())
/// ```
///
/// For tuple structs, prefer `AsSchemaWithSerde` derive:
/// ```
/// # use google_ai_schema_derive::AsSchemaWithSerde;
/// # use google_ai_rs::AsSchema;
/// #[derive(AsSchemaWithSerde)]
/// struct Point(f32, f32);
///
/// ```
/// **Deserialization Note:**  
/// Requires `serde` feature
#[derive(Default)]
pub struct Tuple<T: ?Sized> {
    inner: T,
}

// FIXME: Reduce the indirections here
macro_rules! tuple {
    (
        $(($($T:ident)*))*
    ) => {
        $(impl<$($T, )*> AsSchema for Tuple<($($T, )*)>
        where
            $($T: AsSchema),*
        {
            fn as_schema() -> Schema {
                #[derive(google_ai_schema_derive::AsSchemaWithSerde)]
                #[schema(crate_path = "crate")]
                struct InnerTupleHelper<$($T, )*>($($T, )*);

                #[cfg(feature = "serde")]
                #[allow(non_local_definitions)]
                impl<'de, $($T, )*> serde::Deserialize<'de> for Tuple<($($T, )*)>
                where
                    $($T: serde::Deserialize<'de> + Sized),*
                {
                    #[allow(non_snake_case)]
                    #[inline]
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        // transmuting would have been better
                        let inner = InnerTupleHelper::<$($T, )*>::deserialize(deserializer)?;
                        let InnerTupleHelper($($T, )*) = inner;
                        let inner = ($($T, )*);
                        Ok(Self{inner})
                    }
                }

                InnerTupleHelper::<$($T, )*>::as_schema()
            }
        })*
    };
}

tuple! {
    ()
    (T0)
    (T0 T1)
    (T0 T1 T2)
    (T0 T1 T2 T3)
    (T0 T1 T2 T3 T4)
    (T0 T1 T2 T3 T4 T5)
    (T0 T1 T2 T3 T4 T5 T6)
    (T0 T1 T2 T3 T4 T5 T6 T7)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}

#[cfg(feature = "serde")]
mod serde_support {
    use std::marker::PhantomData;

    use common::{EPhantomData, MapAccessSeqAccess};
    use serde::{de::Visitor, forward_to_deserialize_any, Deserialize, Deserializer};

    use super::{Entry, Map, MapTrait};

    impl<'de, T> Deserialize<'de> for Entry<T>
    where
        T: MapTrait,
        T::Key: Deserialize<'de>,
        T::Value: Deserialize<'de>,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_struct(
                "Entry",
                &[T::KEY_IDENT, T::VALUE_IDENT],
                EPhantomData(PhantomData::<Self>),
            )
        }
    }

    impl<'de, T> Deserialize<'de> for Map<T>
    where
        T: MapTrait + Deserialize<'de>,
    {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct MapToSeq<D, T> {
                inner: D,
                _marker: PhantomData<T>,
            }

            impl<'de, D, T> Deserializer<'de> for MapToSeq<D, T>
            where
                D: Deserializer<'de>,
                T: MapTrait,
            {
                type Error = D::Error;

                fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
                where
                    V: serde::de::Visitor<'de>,
                {
                    struct SeqMapV<V, T> {
                        inner: V,
                        _map: PhantomData<T>,
                    }

                    impl<'de, V, T> Visitor<'de> for SeqMapV<V, T>
                    where
                        V: Visitor<'de>,
                        T: MapTrait,
                    {
                        type Value = V::Value;

                        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(
                                f,
                                "an array of object with two fields representing the key and value of a map"
                            )
                        }

                        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            self.inner.visit_map(MapAccessSeqAccess {
                                _entry: PhantomData::<Entry<T>>,
                                seq,
                            })
                        }
                    }

                    self.inner.deserialize_seq(SeqMapV {
                        inner: visitor,
                        _map: self._marker,
                    })
                }

                forward_to_deserialize_any! {
                    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                    bytes byte_buf option unit unit_struct newtype_struct seq tuple
                    tuple_struct map struct enum identifier ignored_any
                }

                fn is_human_readable(&self) -> bool {
                    self.inner.is_human_readable()
                }
            }

            T::deserialize(MapToSeq {
                inner: deserializer,
                _marker: PhantomData::<T>,
            })
            .map(Into::into)
        }
    }

    #[cfg(test)]
    mod test {
        use std::collections::HashMap;

        use crate::AsSchema;

        use super::*;

        #[test]
        fn map() {
            #[derive(Deserialize, AsSchema, Hash, Eq, PartialEq, Debug)]
            #[schema(crate_path = "crate")]
            struct Question {
                intensity: i64,
                raw: String,
            }

            #[derive(Deserialize, AsSchema, Debug, PartialEq, Eq)]
            #[schema(crate_path = "crate")]
            struct Answer {
                uniqueness: i64,
                raw: String,
            }

            #[derive(PartialEq, Eq, Deserialize, Debug)]
            #[serde(transparent)]
            struct Snippet(HashMap<Question, Answer>);

            impl MapTrait for Snippet {
                type Key = Question;
                type Value = Answer;

                const KEY_IDENT: &str = "question";
                const VALUE_IDENT: &str = "answer";
            }

            assert_eq!(
                Map::<Snippet>::as_schema(),
                Vec::<Entry<Snippet>>::as_schema()
            );

            let response = r#"[{"question": {
                "intensity": 50,
                "raw": "What is the blah blah blah?"
            }, "answer": {
                "uniqueness": 3,
                "raw": "Hmmmm hmm."
            }}]"#;

            let m: Map<Snippet> = serde_json::from_str(response).unwrap();

            assert_eq!(
                m.into_inner(),
                Snippet(
                    [(
                        Question {
                            intensity: 50,
                            raw: "What is the blah blah blah?".into(),
                        },
                        Answer {
                            uniqueness: 3,
                            raw: "Hmmmm hmm.".into(),
                        }
                    )]
                    .into()
                )
            )
        }
    }

    mod common {
        use std::marker::PhantomData;

        use serde::{
            de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
            Deserialize, Deserializer,
        };

        use crate::schema::{Entry, MapTrait};

        pub(super) struct MapAccessSeqAccess<E, S> {
            pub(super) _entry: PhantomData<E>,
            pub(super) seq: S,
        }

        impl<'de, E, S> MapAccess<'de> for MapAccessSeqAccess<E, S>
        where
            E: UnorderedEntry,
            S: SeqAccess<'de>,
        {
            type Error = S::Error;

            fn next_entry_seed<K, V>(
                &mut self,
                kseed: K,
                vseed: V,
            ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
            where
                K: DeserializeSeed<'de>,
                V: DeserializeSeed<'de>,
            {
                self.seq.next_element_seed(UnorderedEntrySeed {
                    key_seed: kseed,
                    value_seed: vseed,
                    _entry: self._entry,
                })
            }

            // The methods below are based on luck. If the key comes before the value, we won't
            // be able to do anything since we don't have the value seed. Besides, we're unable
            // hold on to the mapaccess. So, we just error.
            fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: DeserializeSeed<'de>,
            {
                Err(<Self::Error as serde::de::Error>::custom(
                    "Cannot call next_key_seed on MapAccessSeqAccess. \
                     Use next_entry_seed to process key-value pairs atomically",
                ))
            }

            fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value, Self::Error>
            where
                V: DeserializeSeed<'de>,
            {
                Err(<Self::Error as serde::de::Error>::custom(
                    "Cannot call next_value_seed on MapAccessSeqAccess. \
                     Use next_entry_seed to process key-value pairs atomically",
                ))
            }

            fn size_hint(&self) -> Option<usize> {
                self.seq.size_hint()
            }
        }

        // An entry is seen as a map
        pub(super) struct UnorderedEntrySeed<K, V, E> {
            pub(super) key_seed: K,
            pub(super) value_seed: V,
            pub(super) _entry: PhantomData<E>,
        }

        impl<'de, K, V, E> DeserializeSeed<'de> for UnorderedEntrySeed<K, V, E>
        where
            K: DeserializeSeed<'de>,
            V: DeserializeSeed<'de>,
            E: UnorderedEntry,
        {
            type Value = (K::Value, V::Value);

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_struct(E::NAME, &[E::KEY_IDENT, E::VALUE_IDENT], self)
            }
        }

        impl<'de, K, V, E> Visitor<'de> for UnorderedEntrySeed<K, V, E>
        where
            K: DeserializeSeed<'de>,
            V: DeserializeSeed<'de>,
            E: UnorderedEntry,
        {
            type Value = <Self as DeserializeSeed<'de>>::Value;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a struct representing a map entry")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (mut key, mut value) = (None, None);
                let (mut key_seed, mut value_seed) = (Some(self.key_seed), Some(self.value_seed));
                let (key_ident, value_ident) = (E::KEY_IDENT, E::VALUE_IDENT);

                macro_rules! try_fix_once {
                    ($seed:tt, $target:tt, $field:expr) => {{
                        if $target.is_none() {
                            $target = Some(map.next_value_seed($seed.take().unwrap())?);
                        } else {
                            return Err(serde::de::Error::duplicate_field($field));
                        }
                    }};
                }

                while let Some(field_ident) = map.next_key()? {
                    match field_ident {
                        key_field if key_field == key_ident => {
                            try_fix_once!(key_seed, key, E::KEY_IDENT);
                        }
                        value_field if value_field == value_ident => {
                            try_fix_once!(value_seed, value, E::VALUE_IDENT);
                        }
                        _ => {
                            return Err(<A::Error as serde::de::Error>::unknown_field(
                                field_ident,
                                &[E::KEY_IDENT, E::VALUE_IDENT],
                            ))
                        }
                    }
                }

                match (key, value) {
                    (Some(k), Some(v)) => Ok((k, v)),
                    (None, _) => Err(<A::Error as serde::de::Error>::missing_field(E::KEY_IDENT)),
                    (_, None) => Err(<A::Error as serde::de::Error>::missing_field(
                        E::VALUE_IDENT,
                    )),
                }
            }
        }

        pub(super) trait UnorderedEntry {
            const NAME: &str;
            const KEY_IDENT: &str;
            const VALUE_IDENT: &str;
        }

        impl<T: MapTrait> UnorderedEntry for Entry<T> {
            const NAME: &str = "Entry";
            const KEY_IDENT: &str = T::KEY_IDENT;
            const VALUE_IDENT: &str = T::VALUE_IDENT;
        }

        pub(super) struct EPhantomData<T>(pub PhantomData<T>);

        impl<'de, T> Visitor<'de> for EPhantomData<Entry<T>>
        where
            T: MapTrait,
            T::Key: Deserialize<'de>,
            T::Value: Deserialize<'de>,
        {
            type Value = Entry<T>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a struct representing a map entry")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (key, value) = UnorderedEntrySeed {
                    key_seed: PhantomData,
                    value_seed: PhantomData,
                    _entry: PhantomData::<Entry<T>>,
                }
                .visit_map(map)?;

                Ok(Entry { key, value })
            }
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod derive_test {
    use std::marker::PhantomData;

    use super::AsSchema;

    use crate::{Schema, SchemaType};

    #[test]
    fn rename_all_with() {
        #[derive(AsSchema)]
        #[schema(crate_path = "crate")]
        #[schema(rename_all_with = "sAwCaSe")]
        struct S {
            field: (),
            field1: (),
        }

        #[allow(non_snake_case)]
        fn sAwCaSe(former_name: &str) -> String {
            former_name
                .char_indices()
                .map(|(i, c)| {
                    if i % 2 == 0 {
                        c.to_ascii_lowercase()
                    } else {
                        c.to_ascii_uppercase()
                    }
                })
                .collect()
        }

        let expect = Schema {
            r#type: SchemaType::Object as i32,
            properties: [
                ("fIeLd".into(), <()>::as_schema()),
                ("fIeLd1".into(), <()>::as_schema()),
            ]
            .into(),
            required: vec!["fIeLd".into(), "fIeLd1".into()],
            ..Default::default()
        };

        assert_eq!(S::as_schema(), expect)
    }

    #[test]
    fn as_schema() {
        struct Wrapper<T>(T);

        fn wrapper_as_schema<T: AsSchema>() -> Schema {
            T::as_schema()
        }

        #[derive(AsSchema)]
        #[schema(crate_path = "crate")]
        struct S {
            #[schema(as_schema = "wrapper_as_schema::<String>")]
            field: Wrapper<String>,
        }

        assert_eq!(
            S::as_schema(),
            Schema {
                r#type: SchemaType::Object.into(),
                properties: [("field".into(), String::as_schema())].into(),
                required: vec![("field".into())],
                ..Default::default()
            }
        )
    }

    #[test]
    fn as_schema_generic() {
        struct Wrapper<T>(T);

        fn wrapper_as_schema<T: AsSchema>() -> (Schema, PhantomData<Wrapper<T>>) {
            (T::as_schema(), PhantomData)
        }

        #[derive(AsSchema)]
        #[schema(crate_path = "crate")]
        struct S {
            #[schema(as_schema_generic = "wrapper_as_schema")]
            field: Wrapper<String>,
        }

        assert_eq!(
            S::as_schema(),
            Schema {
                r#type: SchemaType::Object.into(),
                properties: [("field".into(), String::as_schema())].into(),
                required: vec![("field".into())],
                ..Default::default()
            }
        )
    }
}
