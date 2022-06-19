/// A marker trait for marking 2 types not the same.
pub auto trait TypesNotEqual {}
impl<T> !TypesNotEqual for (T, T) {}


