#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use dotenv::dotenv;

mod people;
mod schema;
mod connection;

fn main() {
    dotenv().ok();
    people::router::create_routes();
}
