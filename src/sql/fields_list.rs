use std::marker::PhantomData;

use crate::sql::IntoSqlType;

/// A typed cons list of fields.
pub trait FieldsConsListItem {}

/// A cons item of a typed cons list of fields.
pub struct FieldsConsListCons<
    FieldName: FieldNameCharsConsListItem,
    FieldType,
    Next: FieldsConsListItem,
>(
    PhantomData<FieldName>,
    PhantomData<FieldType>,
    PhantomData<Next>,
);
impl<FieldName: FieldNameCharsConsListItem, FieldType: IntoSqlType, Next: FieldsConsListItem>
    FieldsConsListItem for FieldsConsListCons<FieldName, FieldType, Next>
{
}

/// A typed cons list of field name characters.
pub trait FieldNameCharsConsListItem {}

/// A cons item of a typed cons list of field name characters.
pub struct FieldNameCharsConsListCons<const CHAR: char, Next: FieldNameCharsConsListItem>(
    PhantomData<Next>,
);
impl<const CHAR: char, Next: FieldNameCharsConsListItem> FieldNameCharsConsListItem
    for FieldNameCharsConsListCons<CHAR, Next>
{
}
