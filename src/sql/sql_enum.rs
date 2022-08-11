use std::fmt::Display;

use super::IntoSqlType;
use crate::error::*;

/// An enum consisting of only empty variants, backed up by an integer type,
/// which can be used in sql statements.
pub trait SqlEnum: Sized + 'static + Clone {
    /// The smallest integer type needed to represent this enum.
    type IntegerType: Display + TryInto<usize> + IntoSqlType;

    // The name of the enum.
    const ENUM_NAME: &'static str;

    /// The variants of the enum, in order.
    const VARIANTS_IN_ORDER: &'static [Self];

    /// Converts an integer to a variant of this enum.
    fn from_integer(integer: Self::IntegerType) -> Result<Self> {
        let err = Error::NoSuchEnumVariant {
            enum_name: Self::ENUM_NAME,
            integer_string: integer.to_string(),
        };

        let as_usize: usize = match integer.try_into() {
            Ok(v) => v,
            Err(_) => return Err(err),
        };

        Self::VARIANTS_IN_ORDER.get(as_usize).cloned().ok_or(err)
    }

    /// Converts a variant of this enum to an integer.
    fn to_integer(self) -> Self::IntegerType;
}
