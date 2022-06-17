use gorm::SqlStatement;
use gorm::Table;

fn main() {
    let x = person::age;
}

#[derive(Table)]
#[table(table_name = "person")]
struct Person {
    #[table(primary_key)]
    id: i32,
    name: String,
    age: i32,
}


