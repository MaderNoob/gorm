use gorm::InnerJoinTrait;
use gorm::SelectFrom;
use gorm::Table;

fn main() {
    let x = person::table
        .inner_join(school::table)
        .inner_join(pet::table)
        .find();
}

#[derive(Table)]
#[table(table_name = "person")]
pub struct Person {
    #[table(primary_key)]
    id: i32,
    name: String,
    age: i32,
}

#[derive(Table)]
#[table(table_name = "school")]
pub struct School {
    #[table(primary_key)]
    id: i32,
    name: String,
}

#[derive(Table)]
#[table(table_name = "pet")]
pub struct Pet {
    #[table(primary_key)]
    id: i32,
    name: String,
}
