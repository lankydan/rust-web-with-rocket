#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use diesel::Connection;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

mod people;
mod schema;

fn main() {
    people::router::create_routes();
}

pub fn connect() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
