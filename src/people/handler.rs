use connection::DbConn;
use diesel::result::Error;
use std::env;
use people;
use people::Person;
use rocket::http::Status;
use rocket::response::status;
use rocket_contrib::json::Json;

#[get("/")]
pub fn all(connection: DbConn) -> Result<Json<Vec<Person>>, Status> {
    people::repository::all(&connection)
        .map(|people| Json(people))
        .map_err(|error| error_status(error))
}

fn error_status(error: Error) -> Status {
    match error {
        Error::NotFound => Status::NotFound,
        _ => Status::InternalServerError
    }
}

#[get("/<id>")]
pub fn get(id: i32, connection: DbConn) -> Result<Json<Person>, Status> {
    people::repository::get(id, &connection)
        .map(|person| Json(person))
        .map_err(|error| error_status(error))
}

#[post("/", format = "application/json", data = "<person>")]
pub fn post(person: Json<Person>, connection: DbConn) -> Result<status::Created<Json<Person>>, Status> {
    people::repository::insert(person.into_inner(), &connection)
        .map(|person| person_created(person))
        .map_err(|error| error_status(error))
}

fn person_created(person: Person) -> status::Created<Json<Person>> {
    status::Created(
        format!("{host}:{port}/people/{id}", host = host(), port = port(), id = person.id).to_string(),
        Some(Json(person)))
}

fn host() -> String {
    env::var("ROCKET_ADDRESS").expect("ROCKET_ADDRESS must be set")
}

fn port() -> String {
    env::var("ROCKET_PORT").expect("ROCKET_PORT must be set")
}

#[put("/<id>", format = "application/json", data = "<person>")]
pub fn put(id: i32, person: Json<Person>, connection: DbConn) -> Result<Json<Person>, Status> {
    people::repository::update(id, person.into_inner(), &connection)
        .map(|person| Json(person))
        .map_err(|error| error_status(error))
}

#[delete("/<id>")]
pub fn delete(id: i32, connection: DbConn) -> Result<Status, Status> {
    match people::repository::get(id, &connection) {
        Ok(_) => people::repository::delete(id, &connection)
            .map(|_| Status::NoContent)
            .map_err(|error| error_status(error)),
        Err(error) => Err(error_status(error))
    }
}
