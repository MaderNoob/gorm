use tokio_postgres::types::ToSql;

/// A parameters binder which formats bound parameters as $1 .. $N.
pub struct ParameterBinder<'a> {
    parameters: Vec<&'a (dyn ToSql + Sync)>,
    cur_n: usize,
}
impl<'a> ParameterBinder<'a> {
    pub fn new() -> Self {
        Self {
            cur_n: 1,
            parameters: Default::default(),
        }
    }

    pub fn bind_parameter<T: ToSql + Sync>(
        &mut self,
        parameter: &'a T,
    ) -> DisplayableBoundParameterDollarN {
        self.parameters.push(parameter);

        let result = DisplayableBoundParameterDollarN { n: self.cur_n };

        self.cur_n += 1;

        result
    }

    pub fn parameters(&self) -> &[&'a (dyn ToSql + Sync)] {
        &self.parameters
    }
}

pub struct DisplayableBoundParameterDollarN {
    n: usize,
}
impl std::fmt::Display for DisplayableBoundParameterDollarN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.n)
    }
}
