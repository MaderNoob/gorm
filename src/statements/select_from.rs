use std::marker::PhantomData;

use crate::{
    selectable_tables::{CombineSelectableTables, CombinedSelectableTables, SelectableTables},
    table::{Column, HasForeignKey, Table, TableMarker},
};

use super::SelectStatement;
