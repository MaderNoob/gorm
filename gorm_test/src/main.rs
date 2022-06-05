use gorm::SqlStatement;
use gorm::Table;

fn main() {
    println!("{}", <Person as gorm::Table>::create_table_statement().to_sql_string());
}

#[derive(Table)]
#[table(table_name = "person")]
struct Person {
    #[table(primary_key)]
    id: i32,
    name: String,
    age: i32,
}
