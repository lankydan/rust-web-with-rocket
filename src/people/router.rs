extern crate rocket;

use people;

pub fn create_routes() {
    rocket::ignite()
        .manage(people::connection::init_pool())
        .mount("/people", routes![people::handler::all])
        .mount("/people", routes![people::handler::get])
        .mount("/people", routes![people::handler::post])
        .mount("/people", routes![people::handler::put])
        .mount("/people", routes![people::handler::delete])
        .launch();
}
