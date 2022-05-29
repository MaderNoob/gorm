use gorm::Table;
use gorm_macros::Table;

#[test]
fn basic(){
    #[derive(Debug, Table)]
    struct SomeTable{
        name: String,
        age: i32,
    }
    panic!("{:?}", SomeTable::fields());
}
