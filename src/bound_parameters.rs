use std::fmt::Display;

use sqlx::Database;

#[cfg(feature = "postgres")]
use sqlx::Postgres;

#[cfg(feature = "mssql")]
use sqlx::Mssql;

#[cfg(feature = "mysql")]
use sqlx::MySql;

#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

/// A formatter for bound parameters of some database.
pub trait BoundParametersFormatter {
    /// A type representing a displayable bound parameter.
    type DisplayableBoundParameter: Display;

    /// Creates a new empty bound parameters formatter. A new bound parameters formatter should be
    /// created for each query.
    fn new() -> Self;

    /// Formats the next bound parameter and returns a displayable type for it.
    fn format_next_bound_parameter(&mut self) -> Self::DisplayableBoundParameter;
}

/// A bound parameters formatter for a given database.
pub trait DatabaseBoundParametersFormatter: Database {
    /// The bound parameters formatter for this database.
    type BoundParametersFormatter: BoundParametersFormatter;
}

#[cfg(feature = "postgres")]
impl DatabaseBoundParametersFormatter for Postgres {
    type BoundParametersFormatter = BoundParametersFormatterDollarN;
}

#[cfg(feature = "mssql")]
impl DatabaseBoundParametersFormatter for Mssql {
    type BoundParametersFormatter = BoundParametersFormatterQuestionMark;
}

#[cfg(feature = "mysql")]
impl DatabaseBoundParametersFormatter for MySql {
    type BoundParametersFormatter = BoundParametersFormatterQuestionMark;
}

#[cfg(feature = "sqlite")]
impl DatabaseBoundParametersFormatter for Sqlite {
    type BoundParametersFormatter = BoundParametersFormatterQuestionMark;
}

/// A bound parameters formatter which formats bound parameters as $1 .. $N.
pub struct BoundParametersFormatterDollarN {
    cur_n: usize,
}
impl BoundParametersFormatter for BoundParametersFormatterDollarN {
    type DisplayableBoundParameter = DisplayableBoundParameterDollarN;

    fn new() -> Self {
        Self { cur_n: 1 }
    }

    fn format_next_bound_parameter(&mut self) -> Self::DisplayableBoundParameter {
        let result = DisplayableBoundParameterDollarN { n: self.cur_n };

        self.cur_n += 1;

        result
    }
}
pub struct DisplayableBoundParameterDollarN {
    n: usize,
}
impl std::fmt::Display for DisplayableBoundParameterDollarN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.n.fmt(f)
    }
}

/// A bound parameters formatter which formats bound parameters as '?'.
pub struct BoundParametersFormatterQuestionMark;
impl BoundParametersFormatter for BoundParametersFormatterQuestionMark {
    type DisplayableBoundParameter = char;

    fn new() -> Self {
        Self
    }

    fn format_next_bound_parameter(&mut self) -> Self::DisplayableBoundParameter {
        '?'
    }
}
