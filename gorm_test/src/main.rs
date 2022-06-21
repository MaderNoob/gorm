use gorm::InnerJoinTrait;
use gorm::SelectFrom;
use gorm::Table;

fn main() {
    let x = person::table
        .inner_join(school::table)
        .find();
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
