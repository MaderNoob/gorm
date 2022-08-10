use gorm::{migration, Table};

#[derive(Debug, Table)]
#[table(unique(name, age))]
pub struct Person {
    id: i32,
    name: String,
    age: i32,

    #[table(foreign_key(School))]
    school_id: i32,

    #[table(foreign_key(Pet))]
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

pub struct CreateTablesMigration;
migration! { CreateTablesMigration => school, pet, person }
