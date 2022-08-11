use gorm::{migration, SqlEnum, Table};

#[derive(Debug, Table)]
#[table(unique(name, age))]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age: i32,

    #[table(foreign_key(School))]
    pub school_id: i32,

    #[table(foreign_key(Pet))]
    pub first_pet_id: Option<i32>,

    #[table(foreign_key(Pet))]
    pub second_pet_id: Option<i32>,
}

#[derive(Debug, Table)]
pub struct School {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, SqlEnum, Debug)]
pub enum PetType {
    Cat,
    Dog,
}

#[derive(Debug, Table)]
pub struct Pet {
    pub id: i32,
    pub name: String,
    pub ty: PetType,
}

pub struct CreateTablesMigration;
migration! { CreateTablesMigration => school, pet, person }
