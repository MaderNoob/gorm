//! Utilities that are not directly related to the purpose of this crate.

use crate::sql::{FieldNameCharsConsListItem, FieldsConsListItem};

/// A marker trait for marking 2 types that are not the same type.
/// Used for generic constraints where 2 types must not be equal.
pub auto trait TypesNotEqual {}
impl<T> !TypesNotEqual for (T, T) {}

/// A marker trait for marking 2 types that are the same.
/// Used for generic constraints where 2 types must equal.
pub trait TypesEqual {}
impl<T> TypesEqual for (T, T) {}

/// A nil item of a typed cons list.
pub struct TypedConsListNil;
impl FieldsConsListItem for TypedConsListNil {}
impl FieldNameCharsConsListItem for TypedConsListNil {}

/// A `false` value encoded as a type.
pub struct TypedFalse;

/// A `true` value encoded as a type.
pub struct TypedTrue;

/// A boolean value encoded as a type.
pub trait TypedBool {
    /// The boolean value of the type.
    const VALUE: bool;
}
impl TypedBool for TypedFalse {
    const VALUE: bool = false;
}
impl TypedBool for TypedTrue {
    const VALUE: bool = true;
}
