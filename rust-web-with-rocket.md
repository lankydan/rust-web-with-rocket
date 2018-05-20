Creating a Rusty Rocket fuelled with Diesel

Sorry if that title seemed stupid. The super serious title would be "Creating a REST API in Rust using Rocket and Diesel", but thats boring. Anyway... 

Here I go with my first post that fully focuses on Rust. After spending a few months doing a bit here and there I decided to just dive right in as I was going through the Rust book at too slow a pace to keep myself interested. So, in this post I decided to write about setting up a simple REST API which is something that I have done in Java plenty of times but with Rust it is a different story.

Anyway, enough with this personal backstory and onto the actual tutorial.

In this post we will be looking creating a REST API in Rust. To do this we will use [Rocket](https://rocket.rs/) to setup the API and [Diesel](http://diesel.rs/) to deal with the database. 

At the time of writing this post the only databases that Diesel accommodates are Postgres, MySql and Sqlite.

## Dependencies

Before we can begin coding we need to sort out our dependencies.
```toml
[dependencies]
rocket = "0.3.6"
rocket_codegen = "0.3.6"
diesel = { version = "1.0.0", features = ["postgres"] }
dotenv = "0.9.0"
r2d2-diesel = "1.0"
r2d2 = "0.8"
serde = "1.0"
serde_derive = "1.0"
serde_json = "1.0"

[dependencies.rocket_contrib]
version = "*"
default-features = false
features = ["json"]
```
As you can see there's a reasonable amount of crates being used here. Obviously we need the `rocket` and `diesel` crates whereas the rest are not yet clear. `rocket_codegen` is pulled in for some macros. `dotenv` to allow us to retrieve environment variables from an external file. `r2d2` and `r2d2-diesel` for connection pooling to connect to the database, specifically via diesel. Finally, `serde`, `serde_derive`, `serde_json` for serialisation and deserialisation of data that is sent and received by the REST API. One extra note about the `diesel` dependency, `postgres` has been specified explicitly to include only Postgres modules in the Diesel crate, if we wanted to use a different database or even multiple types within the project we just need to specify them or remove the `features` list all together.

There is one last piece of information we need before we can continue. To use Rocket, we must be using a nightly build of Rust since it relies on features not yet included in the stable builds.

## Doing database stuff with Diesel

I think the best place to start with is setting up Diesel. Once that's done we will have our schema defined (only one table for this post) which we can then use to build up our application.

For the purpose of this post I will assume that you have already setup the Diesel CLI. A quick example on how to use it can be found in Diesel's [getting started guide](http://diesel.rs/guides/getting-started/) along with other information in how to use it. I personally used Postgres solely due to not being able to get everything I needed to run MySQL, which seemed to stem from me using Windows... Postgres on the other hand was nice and easy to get going.

### Creating a table

First set the `DATABASE_URL` to connect to Postgres with the below command or by adding it to the `.env` file manually:
```
echo DATABASE_URL=postgres://postgres:password@localhost/rust-web-with-rocket > .env
```
Hopefully your username and password differs from mine!

Then run `diesel setup` to create a database for the project and an empty migrations folder for later use.

For this post, we will be modelling people who can be: inserted, retrieved, updated and deleted from the database. To do this we are going to first need a table to store them in. So lets create our first migration.
```
diesel migration generate create_people
```
This creates two new files within a single folder which are then placed in the migrations directory. `up.sql` is for upgrading and is where we want to put the SQL to create the table. `down.sql` is for downgrading so we can undo the upgrade if necessary, therefore for this example it will drop the people table.

To create the people table we run:
```sql
CREATE TABLE people(
  id SERIAL PRIMARY KEY,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  age INT NOT NULL,
  profession VARCHAR NOT NULL,
  salary INT NOT NULL
)
```
And to undo this creation:
```sql
DROP TABLE people
```
To apply this migration we need to run:
```
diesel migration run
```
And if we need to undo it right away:
```
diesel migration redo
```
### Mapping to structs

At this point we have a people table which we can start inserting data into. Since Diesel is an ORM we are obviously going to start mapping the table to something that represents the it in Rust. To do just that we will use a struct.
```rust
use super::schema::people;

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
```
Below is the struct that represents each record in the people table; otherwise named a person. Since I only want this struct to represent a record in the table I decided to provide it with no logic and therefore it does not have a `impl` section. There are three Diesel specific attributes here: `#[derive(Queryable)]`, `#[derive(AsChangeSet)]` and `#[table_name]`. `#[derive(Queryable)]` will generate the code to retrieve a person from the database and `#[derive(AsChangeSet)]` to allow us to use `update.set` later on. Finally, `#[table_name = "people"]` is required since the plural of person in not people. If this struct was called post and the table posts, like in the Diesel [getting started example](http://diesel.rs/guides/getting-started/), the attribute can be removed since the plural of post is posts which matches the table name.

The other attributes are aptly named; `#[derive(Serialize)]` and `#[derive(Deserialize)]`. These are for accepting/returning JSON into/from the REST API. They both come from the `serde` crate. We will look at this more later on in the post.

Before we move any further, we should look at creating our schema. Not a database schema for Postgres, a Rust schema file that uses the `table!` macro that does the actual Rust to database mappings for us. If we run the following command:
```
diesel print-schema > src/schema.rs
```
The following file is generated:
```
table! {
    people (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        age -> Int4,
        profession -> Varchar,
        salary -> Int4,
    }
}
```
For now we can just ignore this file and carry on going.

Using the `Person` struct defined above, we can execute `SELECT` and `UPDATE` queries. `DELETE` doesn't require a struct to map to since we just require the record's ID. Then what about `INSERT`? For convenience, Diesel suggests doing it this way, we will use another struct with the sole purpose of being used for inserts.
```rust
#[derive(Insertable)]
#[table_name = "people"]
struct InsertablePerson {
    first_name: String,
    last_name: String,
    age: i32,
    profession: String,
    salary: i32,
}

impl InsertablePerson {

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
```
`InsertablePerson` is nearly identical to the `Person` struct but with one difference, the `id` field is missing. This is because the ID of the record will be generated automatically when inserted, so we have no need to set it ourselves. Other fields could also differ slightly, if we don't want some other fields being set on creation. Similar to the `Person`'s attributes `#[derive(Insertable)]` is added generate the code to insert a new record.

I have also included an utility function `from_person` which takes a `Person` struct's values and converts it into an `InsertablePerson`. This simply removes the `id` field in this scenario and allows me to have tidier code in other places. This function isn't 100% necessary and is added due to my coding preferences.

### Executing queries

At this point we have created our table and the structs that map to it. Now we need to put them to use. Below are all the methods needed to implement the basic REST API:
```rust
use diesel;
use diesel::prelude::*;
use schema::people;
use people::Person;

pub fn all(connection: &PgConnection) -> QueryResult<Vec<Person>> {
    people::table.load::<Person>(&*connection)
}

pub fn get(id: i32, connection: &PgConnection) -> QueryResult<Person> {
    people::table.find(id).get_result::<Person>(connection)
}

pub fn insert(person: Person, connection: &PgConnection) -> QueryResult<Person> {
    diesel::insert_into(people::table)
        .values(&InsertablePerson::from_person(person))
        .get_result(connection)
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
```
The `diesel` module is used to access the `insert_into`, `update` and `delete` functions. `diesel::prelude::*` provides access to a range of modules and structs that are generally useful when using Diesel, for this example; `PgConnection` and `QueryResult` are included in this list. `schema::people` is included so we can access the people table from within Rust and execute methods on it. Note that `schema::people` is referring back to the people table defined in the `schema.rs` file we generated earlier.

Let's look at one of the functions more closely:
```rust
pub fn get(id: i32, connection: &PgConnection) -> QueryResult<Person> {
    people::table.find(id).get_result::<Person>(connection)
}
```
As mentioned above, we can access the people table via `people::table` thanks to including `schema::people`. This example is nice and easy, `find` is specified as the query that selects a single record with the provided ID and `get_result` executes the query with the connection provided to it. 

In my examples `QueryResult` is returned from all functions. Diesel returns `QueryResult&lt;T&gt;` from most methods and is shorthand for `Result<T, Error>` due to the following line:
```rust
pub type QueryResult<T> = Result<T, Error>;
```
Returning `QueryResult` allows us to determine what happens if the query fails in whatever way is suitable for where the function is used. If we wanted to return a `Person` directly out of the function we could call `expect` to log the error there and then.

Also, since I have used Postgres for this post, `PgConnection` is used. If we were using one of the other databases Diesel support; MySql for example, `MysqlConnection` would be used instead.

Let's look at another one:
```rust
pub fn insert(person: Person, connection: &PgConnection) -> QueryResult<Person> {
    diesel::insert_into(people::table)
        .values(&InsertablePerson::from_person(person))
        .get_result(connection)
}
```
This works slightly differently to the earlier `get` function. Rather than accessing a function on the `people::table` it is passed into another Diesel function, `insert_into`. As I mentioned earlier in the post, `InsertablePerson` was defined specifically for new records, therefore the values from `person` are extracted thanks to the `from_person` helper function. Remember that no ID is included on this struct. Like before, `get_result` is called again to execute the statement.

## Connection pooling - A bit of everything

You might have a question following on from the previous section. I'm hoping it's the question I'm about to answer... Where did the `PgConnection` come from? Well, let's have a look.

The code below shows how a connection pool is created:
```rust
use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;
use rocket::{Outcome, Request, State};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use std::env;
use std::ops::Deref;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url());
    Pool::new(manager).expect("db pool")
}

fn database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, Self::Error> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
```
Now, I'm not going to lie. This is a straight up copy from the [Rocket documentation](https://rocket.rs/guide/state/#databases). That link will probably provide a better explanation than I would but I'll give you a quick run through it.  `init_pool` creates a new pool of connections for our database which we have specified as `PgConnection`s. `DbConn` wraps the actual `PgConnection`. Finally, `FromRequest` allows a `DbConn` to be retrieved from Rocket handler functions when included in the input parameters, we will look at an example of this soon.

## Rocket

All of the database magic has been implemented at this point. All we now need to do is create the REST API and hook it up to the back-end that we've created. In Rocket this consists of routes that map incoming requests to handler functions which will then deal with the requests. So we have got two clear things still to do, define the routes and create the handler functions.

### Handlers

It makes sense to start with the handlers first so we actually have an idea of what the routes are mapping to. Below are all the handlers that are needed to implement the typical REST verbs of `GET`, `POST`, `PUT`, `DELETE`:
```rust
use connection::DbConn;
use diesel::result::Error;
use std::env;
use people;
use people::Person;
use rocket::http::Status;
use rocket::response::{Failure, status};
use rocket_contrib::Json;

#[get("/")]
fn all(connection: DbConn) -> Result<Json<Vec<Person>>, Failure> {
    people::repository::all(&connection)
        .map(|people| Json(people))
        .map_err(|error| error_status(error))
}

fn error_status(error: Error) -> Failure {
    Failure(match error {
        Error::NotFound => Status::NotFound,
        _ => Status::InternalServerError
    })
}

#[get("/<id>")]
fn get(id: i32, connection: DbConn) -> Result<Json<Person>, Failure> {
    people::repository::get(id, &connection)
        .map(|person| Json(person))
        .map_err(|error| error_status(error))
}

#[post("/", format = "application/json", data = "<person>")]
fn post(person: Json<Person>, connection: DbConn) -> Result<status::Created<Json<Person>>, Failure> {
    people::repository::insert(person.into_inner(), &connection)
        .map(|person| person_created(person))
        .map_err(|error| error_status(error))
}

fn person_created(person: Person) -> status::Created<Json<Person>> {
    let host = env::var("ROCKET_ADDRESS").expect("ROCKET_ADDRESS must be set");
    let port = env::var("ROCKET_PORT").expect("ROCKET_PORT must be set");
    status::Created(
        format!("{host}:{port}/people/{id}", host = host, port = port, id = person.id).to_string(),
        Some(Json(person)))
}

#[put("/<id>", format = "application/json", data = "<person>")]
fn put(id: i32, person: Json<Person>, connection: DbConn) -> Result<Json<Person>, Failure> {
    people::repository::update(id, person.into_inner(), &connection)
        .map(|person| Json(person))
        .map_err(|error| error_status(error))
}

#[delete("/<id>")]
fn delete(id: i32, connection: DbConn) -> Result<status::NoContent, Failure> {
    match people::repository::get(id, &connection) {
        Ok(_) => people::repository::delete(id, &connection)
            .map(|_| status::NoContent)
            .map_err(|error| error_status(error)),
        Err(error) => Err(error_status(error))
    }
}
```
Each method is marked with an attribute that specifies what REST verb it accepts along with the path needed to get there. Part of the path is missing as the rest will be defined when the routes are created, so just hold on for a bit... The attributes can also accept a few extra properties to properly specify the behavior of the handler. 

Until we look at routing, just assume the base path to these handler methods are `localhost:8000/people`.

Let's look at one of the simpler handlers:
```rust
#[get("/")]
fn all(connection: DbConn) -> Result<Json<Vec<Person>>, Failure> {
    people::repository::all(&connection)
        .map(|people| Json(people))
        .map_err(|error| error_status(error))
}

fn error_status(error: Error) -> Failure {
    Failure(match error {
        Error::NotFound => Status::NotFound,
        _ => Status::InternalServerError
    })
}
```
This function returns all the person records stored in the database. It accepts a `GET` request thanks to the `#[get("/")]` attribute on the function. The path it accepts requests from is `localhost:8000/people` as denoted by the `"/"`.

To use cURL to send a request to this function we need to execute:
```
curl localhost:8000/people
```
This will then return a JSON list of people as specified by the return type of `Result&lt;Json&lt;Vec&lt;Person&gt;&gt;, Failure&gt;`. To do this, records are retrieved from database and mapped into their JSON representation. Thanks to the return type of `QueryResult&lt;Vec&lt;Person&gt;&gt;` from the `all` function, if anything goes wrong at the database level we can then map this to a HTTP status code to represent the error properly. This is why the return type is a `Result`. It provides us with an option to either return the records when nothing goes wrong but if anything does, it can return a error status code instead.

Serde finally pops up here, although it does so behind the scenes. Without the `#[derive(Serialize)]` attribute we added onto the `Person` struct earlier we would not be able to return `Json&lt;Vec&lt;Person&gt;&gt;` from this function; the same applies for `Json&lt;Person&gt;`.

The `error_status` function isn't particularly interesting and doesn't help for this specific example. It simply converts an `Error` contained within `QueryResult` into a status code. I was only particularly interested in these two scenarios, hence why it either returns `NotFound` or `InternalServerError` for anything else since I'm lazy (plus most of the other errors would honestly be classed as internal server errors).

The last point to touch on before we look at another handler function, the appearance of `DbConn`. The code we wrote earlier for connection pooling allows this. At this point all we need to do is include it in the function parameters and it will retrieve a connection for us.

Let's look at the `PUT` handler next:
```rust
#[put("/<id>", format = "application/json", data = "<person>")]
fn put(id: i32, person: Json<Person>, connection: DbConn) -> Result<Json<Person>, Failure> {
    people::repository::update(id, person.into_inner(), &connection)
        .map(|person| Json(person))
        .map_err(|error| error_status(error))
}
```
The first difference between this function and the previous `ALL` example (ignoring request type) is the `id` and `person` being passed in. The `"&lt;/id&gt;"` represents the path variable `id` and `data = "&lt;person"&gt;` represents that request body that maps to `person` in the functions arguments. The `format` property specifies the content of the request body, in other words, the `data` property should contain JSON (indicated by `application/json`). We can see that it does indeed do just that since `person` is of type `Json&lt;Person&gt;`.

Serde again shows up here. It is needed to retrieve the `Json&lt;Person&gt;` from the request body.

To retrieve the contents of `person` we must call `into_inner()`, revealing the `Person` that was waiting to break out all along... `update` is called and the result or error is mapped and returned in the `Result` enum. Due to the implementation of `error_status`, an error will be thrown if an existing record does not exist with the passed in ID. Whether this is how it should work seems to vary from person to person (according to my googling anyway). If we instead wanted to insert the record if it did not already exist, we would need to handle the `Error::NotFound` and instead call similar code to that in the `POST` function.

Well we just mentioned it, so we need to look at it now. Below is the `POST` function:
```rust
#[post("/", format = "application/json", data = "<person>")]
fn post(person: Json<Person>, connection: DbConn) -> Result<status::Created<Json<Person>>, Failure> {
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
```
This contains similar components to the `PUT` function we just looked at. The main difference is the return type. The status code that should be returned from a successful `POST` request is `201 Created` rather than `200 Ok` which was used by the previous functions that we looked at. To return a different status code, the `Result` should contain `status::Created` instead of `Json&lt;Person&gt;` directly. This change is what makes it return a `201` status code.  

To create the `status::Created` struct, the created record along with the path to retrieve it (via a `GET` request) must be passed into it's constructor. Passing in the path as an absolute string isn't ideal so I have retrieved the host and port number from the environment variables. This might not be the best way to get this to work... But I spent ages trying to figure out how to get them out of Rocket and gave up in the end.

We should probably also look at Responders in Rocket and how they enrich the returned responses, but this post is already so long so I will instead refer you to the [Rocket documentation](https://rocket.rs/guide/responses/#rocket-responders) on the subject.

### Routing

We are nearly at the end now... Don't give up yet!

The handlers are setup to accept requests to the server but before we can use them the we need to set the routes to the different functions. Since all of the functions in this post are related to people it will be mapped to `/people`. See the code below on how to do this:
```rust
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
```
`create_routes` is called by the `main` function to get everything rolling. `ignite` creates a new instance of `Rocket`. The handler functions are then mounted onto a base request path of `/people` by specifying all of the them inside of `routes!`. Finally, `launch` starts the application server.

### Configuration

Earlier in this post I made use of environment variables to retrieve the host and port of the running server. So let's have a brief look at the configuration required to change the host and port in Rocket. There are two ways to do this from within configuration files. Either specify values within a `.env` file or create a `Rocket.toml` file. 

When using a `.env` file, the values must follow the format of `ROCKET_{PARAM}` where `PARAM` is the property you are trying to set. `{ADDRESS}` represents the host and `{PORT}` is obviously the port number. Taking this information, below is the `.env` file used in this post (removing unrelated configuration):
```.env
ROCKET_ADDRESS=localhost
ROCKET_PORT=8000
```
If instead you wanted to use a `Rocket.toml` file, it would look like the below.
```toml
[development]
address = "localhost"
port = 8000
```
In this situation, these values are only applicable for development, which is handy since thats all I'm doing.

If you choose to include neither of these Rocket will instead fall back to it's default configuration. So don't worry about needing to do loads of configuration when playing around with Rocket; for local development the defaults are most likely good enough.

For more (and better) explanations of Rocket Configuration, I again recommend looking at their [documentation](https://rocket.rs/guide/configuration/#configuration).

## The last step

Finally we have reached the end. All that is left to do now is create the `main` method so the application can be run.
```rust
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
```
All `main` does is load in the environment variables and starts Rocket by calling `create_routes`. The rest of this file just pulls in a load of crates so they don't need to be scattered throughout the rest of the code.

Now you can rest. That was a pretty long post. I'd write a conclusion but honestly, I'm tired and don't want to write anymore. So for a short summary, in this post we have created a simple REST API using Rocket to run an application server that responds to requests and used Diesel to connect to a database to manage the state of the application.

The code used in this post can be on my [GitHub](https://github.com/lankydan/rust-web-with-rocket).

If you liked this post, then follow me on Twitter at [@LankyDanDev](https://twitter.com/LankyDanDev) to be able to keep up with my new posts as I write them.

