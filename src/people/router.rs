use people;
use rocket;
use connection;

pub fn create_routes() {
    rocket::ignite()
        .manage(connection::init_pool())
        .mount("/people",
               routes![people::handler::all,
                    people::handler::get,
                    people::handler::post,
                    people::handler::put,
                    people::handler::delete],
        ).launch();
}
