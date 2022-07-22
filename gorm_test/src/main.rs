use gorm::expr::SqlExpression;
use gorm::connection::DatabaseConnection;
use gorm::table::TableMarker;
use gorm::ExecuteSqlStatment;
use gorm::SelectFrom;
use gorm::Table;

#[tokio::main]
async fn main() {
    let client = DatabaseConnection::connect(
        "postgres://postgres:postgres@localhost/gorm_test",
    )
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
        .find()
        .filter(person::name.eq("avi"))
        .load_all::<Person>(&client)
        .await
        .unwrap();
    println!("{:?}", p);
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
