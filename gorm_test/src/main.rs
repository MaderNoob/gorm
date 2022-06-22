use gorm::pool::DatabaseConnectionPool;
use gorm::sqlx::Postgres;
use gorm::table::TableMarker;
use gorm::ExecuteSqlStatment;
use gorm::SelectFrom;
use gorm::SqlStatement;
use gorm::Table;

#[tokio::main]
async fn main() {
    let pool = DatabaseConnectionPool::<Postgres>::connect(
        "postgres://postgres:postgres@localhost/gorm_test",
    )
    .await
    .unwrap();
    school::table
        .create()
        .if_not_exists()
        .execute(&pool)
        .await
        .unwrap();
    person::table
        .create()
        .if_not_exists()
        .execute(&pool)
        .await
        .unwrap();
    //let query = person::table.inner_join(school::table).find();
    let query = person::table.find();
    println!("{}", query.formatter());
}

#[derive(Table)]
pub struct Person {
    id: i32,
    name: String,
    age: i32,

    #[table(foreign_key = "School")]
    school_id: i32,
}

#[derive(Table)]
pub struct School {
    id: i32,
    name: String,
}

#[derive(Table)]
pub struct Pet {
    id: i32,
    name: String,
}
