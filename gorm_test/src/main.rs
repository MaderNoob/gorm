use gorm::{
    execution::DatabaseConnectionPool,
    migration, returning, select_values, selected_value_to_order_by,
    sql::{
        self, AddableSqlExpression, BooleanAndableSqlExpression, BooleanOrableSqlExpression,
        Insertable, Migration, MultipliableSqlExpression, OrderableSqlExpression, SqlExpression,
        SummableSqlExpression, TableMarker,
    },
    statements::{
        ExecuteSqlStatment, Filter, GroupBy, InnerJoinTrait, LoadSingleColumnSqlStatment,
        LoadSqlStatment, OrderBy, OrderBySelectedValue, SelectFrom, SelectValues,
    },
    FromQueryResult, Table,
};
struct CreateTablesMigration;
migration! {CreateTablesMigration => school, pet, person}

#[tokio::main]
async fn main() {
    let pool = DatabaseConnectionPool::connect("postgres://postgres:postgres@localhost/gorm_test")
        .await
        .unwrap();

    {
        let mut client = pool.get().await.unwrap();
        let transaction = client.begin_transaction().await.unwrap();

        CreateTablesMigration::down(&pool).await.unwrap();
        CreateTablesMigration::up(&transaction).await.unwrap();

        transaction.commit().await.unwrap();
    }

    school::new { name: "mekif" }.insert(&pool).await.unwrap();

    let pet_id = pet::new_with_id {
        name: "Kitty",
        id: &5,
    }
    .insert_returning_value(returning!(pet::id), &pool)
    .await
    .unwrap();

    person::new {
        name: "James",
        age: &16,
        school_id: &1,
        pet_id: &None,
    }
    .insert(&pool)
    .await
    .unwrap();

    #[derive(Debug, FromQueryResult)]
    struct NewPersonInfo {
        id: i32,
    }
    let new_person_info = person::new {
        name: "Avi",
        age: &17,
        school_id: &1,
        pet_id: &Some(pet_id),
    }
    .insert_returning::<Person>(person::all, &pool)
    .await
    .unwrap();

    println!("new person info: {:?}", new_person_info);

    // let deleted_people_ids = person::table
    //     .delete()
    //     .filter(person::id.lower_than(10))
    //     .returning(returning!(person::id))
    //     .load_all_values(&pool)
    //     .await
    //     .unwrap();
    // println!("deleted people: {:?}", deleted_people_ids);

    #[derive(Debug, FromQueryResult)]
    struct PersonNameAndSchoolName {
        name: String,
        school_name: String,
    }

    let p = person::table
        .inner_join(school::table)
        .find()
        .filter(
            school::id
                .greater_equals(1)
                .or(school::name.equals("mekif").and(school::id.greater_than(2))),
        )
        .select(select_values!(person::name, school::name as school_name))
        .order_by_ascending(person::name)
        .load_all::<PersonNameAndSchoolName>(&pool)
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
        .filter(person::age.greater_than(0))
        .group_by(person::school_id.add(person::id))
        .order_by_selected_value_descending(selected_value_to_order_by!(avg_age))
        .load_all::<PeopleAvgAge>(&pool)
        .await
        .unwrap();

    println!("{:?}", p);

    let people = person::table
        .find()
        .select(person::all)
        .load_all::<Person>(&pool)
        .await
        .unwrap();

    println!("people: {:?}", people);

    #[derive(Debug, FromQueryResult)]
    struct PersonAndPetName {
        name: String,
        pet_name: String,
    }

    let p = person::table
        .inner_join(pet::table)
        .find()
        .select(select_values!(person::name, pet::name as pet_name))
        .load_all::<PersonAndPetName>(&pool)
        .await
        .unwrap();

    println!("{:?}", p);

    let p = person::table
        .find()
        .select(select_values!(person::name))
        .load_all_values(&pool)
        .await
        .unwrap();
    println!("{:?}", p);

    let count = person::table
        .find()
        .select(select_values!(sql::count_rows() as count))
        .load_one_value(&pool)
        .await
        .unwrap();

    println!("count: {}", count);
}

#[derive(Debug, Table)]
pub struct Person {
    id: i32,
    name: String,
    age: i32,

    #[table(foreign_key = "School")]
    school_id: i32,

    #[table(foreign_key = "Pet")]
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
