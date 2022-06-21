use gorm::table::TableMarker;
use gorm::InnerJoinTrait;
use gorm::SelectFrom;
use gorm::SqlStatement;
use gorm::Table;

fn main() {
    let c2 = school::table.create();
    println!("{}", c2.formatter());
    let c = person::table.create();
    println!("{}", c.formatter());
    let query = person::table.inner_join(school::table).find();
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
