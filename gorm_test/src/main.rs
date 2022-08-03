use gorm::{
    execution::DatabaseConnection,
    migration, select_values,
    sql::{
        AddableSqlExpression, AverageableSqlExpression, Insertable, Migration,
        MultipliableSqlExpression, OrderableSqlExpression, SqlExpression, SummableSqlExpression,
        TableMarker,
    },
    statements::{ExecuteSqlStatment, InnerJoinTrait, SelectFrom},
    Decimal, FromQueryResult, Table,
};

struct CreateTablesMigration;
migration! {CreateTablesMigration => school, pet, person}

#[tokio::main]
async fn main() {
    let client = DatabaseConnection::connect("postgres://postgres:postgres@localhost/gorm_test")
        .await
        .unwrap();

    CreateTablesMigration::up(&client).await.unwrap();

    person::new {
        name: "James".to_string(),
        age: 16,
        school_id: 1,
        pet_id: None,
    }
    .insert(&client)
    .await
    .unwrap();

    #[derive(Debug, FromQueryResult)]
    struct PersonNameAndSchoolName {
        name: String,
        school_name: String,
    }

    let p = person::table
        .inner_join(school::table)
        .find()
        .filter(school::id.greater_equals(1))
        .select(select_values!(person::name, school::name as school_name))
        .load_all::<PersonNameAndSchoolName>(&client)
        .await
        .unwrap();

    println!("{:?}", p);

    #[derive(Debug, FromQueryResult)]
    struct PeopleAvgAge {
        avg_age: i64,
    }

    let p = person::table
        .find()
        .select(select_values!(
            person::age.multiply(person::id).sum() as avg_age
        ))
        .group_by(person::school_id.add(person::id))
        .load_all::<PeopleAvgAge>(&client)
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

    pet_id: Option<i32>,
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
