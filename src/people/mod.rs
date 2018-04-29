use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use super::schema::people;

pub mod connection;
pub mod handler;
pub mod router;
pub mod repository;

#[derive(Queryable, AsChangeset, Serialize, Deserialize)]
#[table_name = "people"]
pub struct Person {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub profession: String,
    pub salary: i32,
}

impl Person {
    pub fn all(connection: &PgConnection) -> QueryResult<Vec<Person>> {
        people::table.load::<Person>(&*connection)
    }

    pub fn get(id: i32, connection: &PgConnection) -> QueryResult<Person> {
        people::table.find(id).get_result::<Person>(connection)
    }

    pub fn update(id: i32, person: Person, connection: &PgConnection) -> QueryResult<Person> {
        diesel::update(people::table.find(id))
            .set(&person)
            .get_result(connection)
    }

    pub fn delete(id: i32, connection: &PgConnection) -> QueryResult<usize> {
        diesel::delete(people::table.find(id))
            .execute(connection)
    }
}

#[derive(Insertable)]
#[table_name = "people"]
pub struct InsertablePerson {
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub profession: String,
    pub salary: i32,
}

impl InsertablePerson {

    pub fn insert(person: Person, connection: &PgConnection) -> QueryResult<Person> {
        diesel::insert_into(people::table)
            .values(&InsertablePerson::from_person(person))
            .get_result(connection)
    }

    fn from_person(person: Person) -> InsertablePerson {
        InsertablePerson {
            first_name: person.first_name,
            last_name: person.last_name,
            age: person.age,
            profession: person.profession,
            salary: person.salary,
        }
    }
}
