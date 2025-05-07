use std::{
    borrow::Cow, cell::{Cell, RefCell, RefMut}, collections::{BTreeSet, BinaryHeap, HashSet, LinkedList, VecDeque}, ffi::{CStr, CString}, hash::Hash, num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    }, path::{Path, PathBuf}, rc::{Rc, Weak}, sync::{
        atomic::{
            AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16,
            AtomicU32, AtomicU64, AtomicU8, AtomicUsize,
        },
        Arc, Mutex, RwLock, Weak as ArcWeak,
    }
};
use tokio::sync::{Mutex as TMutex, RwLock as TRwLock};

use crate::proto::{Schema, Type};

// SchemaType contains the list of OpenAPI data types as defined by
// https://spec.openapis.org/oas/v3.0.3#data-types
pub type SchemaType = Type;

/// Trait for Rust types that can generate `Schema` (a subset of OpenAPI schemas) automatically.
/// 
/// Implement this trait or derive `AsSchema` to enable schema generation for your types.
/// The derive macro supports extensive customization through attributes and integrates with Serde.
/// 
/// # Examples
/// 
/// ```rust
/// use google_ai_rs::AsSchema;
///
/// #[derive(AsSchema)]
/// #[schema(rename_all = "camelCase")]
/// struct AiReport {
///     #[schema(description = "This should include user's name and date of birth", required)]
///     data: String,
/// }
/// ```
/// 
/// For foreign types, manually implement or use schema attributes:
/// ```rust
/// use google_ai_rs::AsSchema;
/// # mod some_crate {
/// #    pub struct TheirType;
/// # }
///
/// #[derive(AsSchema)]
///  struct AiReport {
///     #[schema(r#type = "Number", format = "double")]    
///     foreign: some_crate::TheirType
/// }
/// ```
///
/// ```rust
/// use google_ai_rs::AsSchema;
/// # mod some_crate {
/// #    pub struct TheirType;
/// # }
///
/// #[derive(AsSchema)]
///  struct AiReport {
///     #[schema(as_schema = "foreign_type_schema")]
///     foreign: some_crate::TheirType
/// }
///
/// use google_ai_rs::Schema;
///
/// fn foreign_type_schema() -> Schema {
///     # stringify! {
///     ...
///     # };
///     # unimplemented!()   
/// }
/// ```
/// # Serde Compatibility
/// 
/// - `#[serde(rename)]`/`#[serde(rename_all)]` are automatically respected
/// - `#[serde(skip)]` fields are excluded from schemas by default
/// - Disable Serde integration with `#[schema(ignore_serde)]`
/// 
/// # Examples
/// 
/// ```rust
/// use google_ai_rs::AsSchema;
///
/// #[derive(AsSchema, serde::Deserialize)]
/// struct AiReport {
///     #[schema(description = "Important field", required)]
///     #[serde(rename = "json_field")]  // Applies to schema too
///     field: String,
///
///     #[serde(skip)]  // Excluded from schema
///     internal: String,
///
///     #[schema(skip)]  // Override Serde behavior
///     #[serde(rename = "count")]  // Ignored by schema
///     item_count: i32,
/// }
/// ```
#[cfg_attr(
    not(no_diagnostic_namespace),
    diagnostic::on_unimplemented(
        note = "for local types consider adding `#[derive(google_ai_rs::AsSchema)]` to your `{Self}` type",
        note = "for types from other crates consider the as_schema attribute or check if you can represent the type with the r#type and format attribute",
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
	            #[inline]
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
    RefMut<'a, T>
}

impl<'a, T: AsSchema + ToOwned + ?Sized + 'a> AsSchema for Cow<'a, T> {
    fn as_schema() -> Schema {
        T::as_schema()
    }
}

macro_rules! number {
    ($($n:ident, $ty:ident, $format:expr)*) => {
        $(impl AsSchema for $n {
            #[inline]
            fn as_schema() -> Schema {
                Schema {
		            r#type: SchemaType::$ty as i32,
		            format: $format.into(),
		            ..Default::default()
		        }
            }
        })*
    };
}

// These are positive integers. But, integer
// is wider so we can't strictly call them
// integers. It'd be our fault if the model
// outputs -2
number! {
    usize, Number, ""
    u8, Number, ""
    u16, Number, ""
    u32, Number, ""
    u64, Number, ""
    u128, Number, ""
    AtomicUsize, Number, ""
    AtomicU8, Number, ""
    AtomicU16, Number, ""
    AtomicU32, Number, ""
    AtomicU64, Number, ""
    NonZeroUsize, Number, ""
    NonZeroU8, Number, ""
    NonZeroU16, Number, ""
    NonZeroU32, Number, ""
    NonZeroU64, Number, ""
    NonZeroU128, Number, ""
}

number! {
    isize, Integer, ""
    i8, Integer, ""
    i16, Integer, ""
    i32, Integer, "int32"
    i64, Integer, "int64"
    i128, Integer, ""
    AtomicIsize, Integer, ""
    AtomicI8, Integer, ""
    AtomicI16, Integer, ""
    AtomicI32, Integer, "int32"
    AtomicI64, Integer, "int64"
    NonZeroIsize, Integer, ""
    NonZeroI8, Integer, ""
    NonZeroI16, Integer, ""
    NonZeroI32, Integer, "int32"
    NonZeroI64, Integer, "int64"
    NonZeroI128, Integer, ""
}

number! {
    f32, Number, "Float"
    f64, Number, "double"
}

macro_rules! string {
    ($($n:ident)*) => {
    	$(
        impl AsSchema for $n {
            #[inline]
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
	            #[inline]
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
    HashSet<T: Eq + Hash>
    BTreeSet<T: Ord>
    BinaryHeap<T: Ord>
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

// Represent tuples as struct and give the corresponding serde deserializer using feature?
