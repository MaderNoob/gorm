use deadpool_postgres::tokio_postgres::types::ToSql;

/// A parameters binder which formats bound parameters as $1 .. $N and saves the
/// actual values in a list.
pub struct ParameterBinder<'a> {
    parameters: Vec<&'a (dyn ToSql + Sync)>,
    cur_n: usize,
}
impl<'a> ParameterBinder<'a> {
    /// Creates a new parameter binder with an empty list of values, and which
    /// starts from $1.
    ///
    /// A new parameter binder should be created for each statement.
    pub fn new() -> Self {
        Self {
            cur_n: 1,
            parameters: Default::default(),
        }
    }

    /// Binds the given parameter, adding it to the list of values, and returns
    /// a [`DisplayableBoundParameterDollarN`], which allows allows
    /// formatting the sql identifier of this parameter ($1..$N).
    pub fn bind_parameter<T: ToSql + Sync>(
        &mut self,
        parameter: &'a T,
    ) -> DisplayableBoundParameterDollarN {
        self.parameters.push(parameter);

        let result = DisplayableBoundParameterDollarN { n: self.cur_n };

        self.cur_n += 1;

        result
    }

    /// Returns a reference to the list of parameters bound to this parameter binder.
    pub fn parameters(&self) -> &[&'a (dyn ToSql + Sync)] {
        &self.parameters
    }
}

/// A struct which implements the [`std::fmt::Display`] trait and allows formatting a bound
/// parameter's sql identifier ($1..$N).
pub struct DisplayableBoundParameterDollarN {
    n: usize,
}
impl std::fmt::Display for DisplayableBoundParameterDollarN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.n)
    }
}
