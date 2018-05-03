use people;
use rocket;
use connection;

pub fn create_routes() {
    rocket::ignite()
        .manage(connection::init_pool())
        .mount("/people", routes![people::handler::all])
        .mount("/people", routes![people::handler::get])
        .mount("/people", routes![people::handler::post])
        .mount("/people", routes![people::handler::put])
        .mount("/people", routes![people::handler::delete])
        .launch();
}
