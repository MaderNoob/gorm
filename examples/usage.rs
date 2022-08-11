use gorm::{
    execution::DatabaseConnectionPool,
    returning, select_values, selected_value_to_order_by,
    sql::{
        self, AddableSqlExpression, BooleanAndableSqlExpression, BooleanOrableSqlExpression,
        Insertable, LikeableSqlExpression, Migration, MultipliableSqlExpression,
        OrderableSqlExpression, SqlExpression, TableMarker,
    },
    statements::{
        DeleteStatementReturning, ExecuteSqlStatment, Filter, FilterDeleteStatement, GroupBy,
        InnerJoinOnTrait, InnerJoinTrait, InsertStatementOnConflict, InsertStatementReturning,
        LoadSingleColumnSqlStatment, LoadSqlStatment, OrderBy, OrderBySelectedValue, SelectFrom,
        SelectValues, UpdateStatementReturning,
    },
    update_set, FromQueryResult,
};

mod tables;
use tables::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool =
        DatabaseConnectionPool::connect("postgres://postgres:postgres@localhost/gorm_test").await?;

    // it is recommended to scope the lifetime of a client we get from the pool, so
    // that the client will be returned to the pool as soon as possible.
    //
    // there isn't really any point in using a transaction here, but this is just to
    // show how to get connections from connection pools, and how to use
    // transactions.
    {
        let mut client = pool.get().await?;
        let transaction = client.begin_transaction().await?;

        CreateTablesMigration::down(&transaction).await?;
        CreateTablesMigration::up(&transaction).await?;

        transaction.commit().await?;
    }

    school::new { name: "Stanford" }
        .insert()
        .execute(&pool)
        .await?;

    let second_school = school::new {
        name: "Marble Hills School",
    }
    .insert()
    .returning(school::id)
    .load_one_value(&pool)
    .await?;

    let inserted_pet = pet::new_with_id {
        name: "Kitty",
        id: &5,
        ty: &PetType::Dog,
    }
    .insert()
    .returning(pet::all)
    .load_one::<Pet>(&pool)
    .await?;

    println!("inserted pet: {:?}", inserted_pet);

    person::new {
        name: "James",
        age: &44,
        school_id: &1,
        first_pet_id: &None,
        second_pet_id: &None,
    }
    .insert()
    .execute(&pool)
    .await?;

    person::new {
        name: "Harry",
        age: &33,
        school_id: &1,
        first_pet_id: &None,
        second_pet_id: &None,
    }
    .insert()
    .execute(&pool)
    .await?;

    person::new {
        name: "David",
        age: &34,
        school_id: &1,
        first_pet_id: &None,
        second_pet_id: &None,
    }
    .insert()
    .execute(&pool)
    .await?;

    let upserted_person = person::new {
        name: "James",
        age: &44,
        school_id: &second_school,
        first_pet_id: &None,
        second_pet_id: &None,
    }
    .insert()
    .on_conflict(person::unique_constraints::name_age)
    .do_update(update_set!(person::school_id = second_school))
    .returning(person::all)
    .load_one::<Person>(&pool)
    .await?;

    // try insert, and if there is a conflict, do nothing
    let person_if_no_conflict_occured = person::new {
        name: "James",
        age: &44,
        school_id: &second_school,
        first_pet_id: &None,
        second_pet_id: &None,
    }
    .insert()
    .on_conflict_do_nothing()
    .returning(person::all)
    .load_optional::<Person>(&pool)
    .await?;

    println!("person if no conflict occured: {:?}", person_if_no_conflict_occured);

    let inserted_person = person::new {
        name: "Jake",
        age: &29,
        school_id: &1,
        first_pet_id: &Some(inserted_pet.id),
        second_pet_id: &None,
    }
    .insert()
    .returning(person::all)
    .load_one::<Person>(&pool)
    .await?;

    println!("inserted person: {:?}", inserted_person);

    let deleted_people_ids = person::table
        .delete()
        .filter(person::name.not_like("J%"))
        .returning(returning!(person::id))
        .load_all_values(&pool)
        .await
        .unwrap();
    println!("deleted people: {:?}", deleted_people_ids);

    #[derive(Debug, FromQueryResult)]
    struct PersonNameAndSchoolName {
        name: String,
        school_name: String,
    }

    let people_and_school_names = person::table
        .inner_join(school::table)
        .find()
        .filter(
            // This conditions is overcomplicated, but its purpose is to show how conditions can be
            // combined together to form more complicated conditions.
            school::id.greater_equals(1).or(school::name
                .equals("Stanford")
                .and(school::id.greater_than(2))),
        )
        .select(select_values!(person::name, school::name as school_name))
        .order_by_ascending(person::name)
        .load_all::<PersonNameAndSchoolName>(&pool)
        .await
        .unwrap();

    println!("{:?}", people_and_school_names);

    #[derive(Debug, FromQueryResult)]
    struct SomeAggregateExpression {
        some_aggregate_expression: i64,
    }

    // this shows how you can use complicated aggregate expressions, and combine
    // many query functions together to form a complicated query.
    let aggregate_exprs = person::table
        .find()
        .select(select_values!(
            sql::sum(person::age.multiply(person::id)) as some_aggregate_expression
        ))
        .filter(person::age.greater_than(0))
        .group_by(person::school_id.add(person::id))
        .order_by_selected_value_descending(selected_value_to_order_by!(some_aggregate_expression))
        .load_all::<SomeAggregateExpression>(&pool)
        .await
        .unwrap();

    println!("aggregate expressions: {:?}", aggregate_exprs);

    // this shows how you can use the `all` struct to select all fields of some
    // table. In this example using `all` doesn't make much sense because we
    // will get repetition in the results, since multiple people have the same
    // `school_id`, but this is just for the sake of showing how it can be used.
    let schools_of_people = person::table
        .inner_join(school::table)
        .find()
        .select(school::all)
        .load_all::<School>(&pool)
        .await
        .unwrap();

    println!("people: {:?}", schools_of_people);

    #[derive(Debug, FromQueryResult)]
    struct PersonAndPetName {
        name: String,
        pet_name: String,
    }

    // this shows how you can perform inner joins on optional foreign keys, which
    // will return only the records whose foreign key column isn't `NULL`.
    let person_and_pet_names = person::table
        .inner_join_on_column(person::first_pet_id, pet::table)
        .find()
        .select(select_values!(person::name, pet::name as pet_name))
        .load_all::<PersonAndPetName>(&pool)
        .await
        .unwrap();

    println!("{:?}", person_and_pet_names);

    // This shows how you can load values in case you only want one column instead
    // of parsing into a struct.
    let names = person::table
        .find()
        .select(select_values!(person::name))
        .load_all_values(&pool)
        .await
        .unwrap();
    println!("{:?}", names);

    let count = person::table
        .find()
        .select(select_values!(sql::count_rows() as count))
        .load_one_value(&pool)
        .await
        .unwrap();

    println!("count: {}", count);

    let ages = person::table
        .update()
        .set(update_set!(person::age = person::age.add(1)))
        .returning(person::age)
        .load_all_values(&pool)
        .await?;

    println!("updated ages: {:?}", ages);

    Ok(())
}
