use crate::sql::{FieldNameCharsConsListItem, FieldsConsListItem};

/// A marker trait for marking 2 types not the same.
pub auto trait TypesNotEqual {}
impl<T> !TypesNotEqual for (T, T) {}

pub trait TypesEqual {}
impl<T> TypesEqual for (T, T) {}

/// A nil item of a types cons list.
pub struct TypedConsListNil;
impl FieldsConsListItem for TypedConsListNil {}
impl FieldNameCharsConsListItem for TypedConsListNil {}

pub struct TypedFalse;
pub struct TypedTrue;

pub trait TypedBool {
    const VALUE: bool;
}
impl TypedBool for TypedFalse {
    const VALUE: bool = false;
}
impl TypedBool for TypedTrue {
    const VALUE: bool = true;
}
