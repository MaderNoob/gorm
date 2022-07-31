use std::marker::PhantomData;

use gorm::expr::OrderableSqlExpression;
use gorm::expr::SqlExpression;
use gorm::table::TableMarker;
use gorm::ExecuteSqlStatment;
use gorm::SelectFrom;
use gorm::Table;
use gorm::{
    connection::DatabaseConnection, select_values, selectable_tables::CombinedSelectableTables,
    InnerJoined,
};
use gorm::{
    selectable_tables::SelectableTables, selected_values::SelectedValues, InnerJoinTrait,
    TypedConsListNil,
};

#[tokio::main]
async fn main() {
    let client = DatabaseConnection::connect("postgres://postgres:postgres@localhost/gorm_test")
        .await
        .unwrap();

    school::table
        .create()
        .if_not_exists()
        .execute(&client)
        .await
        .unwrap();

    person::table
        .create()
        .if_not_exists()
        .execute(&client)
        .await
        .unwrap();

    let p = person::table
        .inner_join(school::table)
        .find()
        .filter(school::id.greater_equals(1))
        .load_all::<Person>(&client)
        .await
        .unwrap();

    println!("{:?}", p);

    let s = select_values!(5 as david, person::name);
}

#[derive(Debug, Table)]
pub struct Person {
    id: i32,
    name: String,
    age: i32,

    #[table(foreign_key = "School")]
    school_id: i32,
}

#[derive(Debug, Table)]
pub struct School {
    id: i32,
    name: String,
}

#[derive(Table)]
pub struct Pet {
    id: i32,
    name: String,
}
