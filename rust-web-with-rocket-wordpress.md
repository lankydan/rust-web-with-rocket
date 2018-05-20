Sorry if that title seemed stupid. The super serious title would be "Creating a REST API in Rust using Rocket and Diesel", but thats boring. Anyway... 

Here I go with my first post that fully focuses on Rust. After spending a few months doing a bit here and there I decided to just dive right in as I was going through the Rust book at too slow a pace to keep myself interested. So, in this post I decided to write about setting up a simple REST API which is something that I have done in Java plenty of times but with Rust it is a different story.

Anyway, enough with this personal backstory and onto the actual tutorial.

In this post we will be looking creating a REST API in Rust. To do this we will use<a href="https://rocket.rs/" target="_blank" rel="noopener">Rocket</a> to setup the API and <a href="http://diesel.rs/" target="_blank" rel="noopener">Diesel</a> to deal with the database. 

At the time of writing this post the only databases that Diesel accommodates are Postgres, MySql and Sqlite.

<h2>Dependencies</h2>
Before we can begin coding we need to sort out our dependencies.

[gist https://gist.github.com/lankydan/2b72b641b905ff37af99a512fa5638bd /]

As you can see there's a reasonable amount of crates being used here. Obviously we need the <code>rocket</code> and <code>diesel</code> crates whereas the rest are not yet clear. <code>rocket_codegen</code> is pulled in for some macros. <code>dotenv</code> to allow us to retrieve environment variables from an external file. <code>r2d2</code> and <code>r2d2-diesel</code> for connection pooling to connect to the database, specifically via diesel. Finally, <code>serde</code>, <code>serde_derive</code>, <code>serde_json</code> for serialisation and deserialisation of data that is sent and received by the REST API. One extra note about the <code>diesel</code> dependency, <code>postgres</code> has been specified explicitly to include only Postgres modules in the Diesel crate, if we wanted to use a different database or even multiple types within the project we just need to specify them or remove the <code>features</code> list all together.

There is one last piece of information we need before we can continue. To use Rocket, we must be using a nightly build of Rust since it relies on features not yet included in the stable builds.
<h2>Doing database stuff with Diesel</h2>
I think the best place to start with is setting up Diesel. Once that's done we will have our schema defined (only one table for this post) which we can then use to build up our application.

For the purpose of this post I will assume that you have already setup the Diesel CLI. A quick example on how to use it can be found in Diesel's <a href="http://diesel.rs/guides/getting-started/" target="_blank" rel="noopener">getting started guide</a> along with other information in how to use it. I personally used Postgres solely due to not being able to get everything I needed to run MySQL, which seemed to stem from me using Windows... Postgres on the other hand was nice and easy to get going.
<h3>Creating a table</h3>
First set the <code>DATABASE_URL</code> to connect to Postgres with the below command or by adding it to the <code>.env</code> file manually:
<pre>
echo DATABASE_URL=postgres://postgres:password@localhost/rust-web-with-rocket > .env
</pre>
Hopefully your username and password differs from mine!

Then run <code>diesel setup</code> to create a database for the project and an empty migrations folder for later use.

For this post, we will be modelling people who can be: inserted, retrieved, updated and deleted from the database. To do this we are going to first need a table to store them in. So lets create our first migration.
<pre>
diesel migration generate create_people
</pre>
This creates two new files within a single folder which are then placed in the migrations directory. <code>up.sql</code> is for upgrading and is where we want to put the SQL to create the table. <code>down.sql</code> is for downgrading so we can undo the upgrade if necessary, therefore for this example it will drop the people table.

To create the people table we run:

[gist https://gist.github.com/lankydan/40800defe7f02485988bddb5b97477a3 /]

And to undo this creation:

[gist https://gist.github.com/lankydan/7b7bbaa18e8ea1e05fe754f5e4d9f179 /]

To apply this migration we need to run:
<pre>
diesel migration run
</pre>
And if we need to undo it right away:
<pre>
diesel migration redo
</pre>
<h3>Mapping to structs</h3>
At this point we have a people table which we can start inserting data into. Since Diesel is an ORM we are obviously going to start mapping the table to something that represents the it in Rust. To do just that we will use a struct.

[gist https://gist.github.com/lankydan/6ca1eb5d800ce430432e476061a88034 /]

Below is the struct that represents each record in the people table; otherwise named a person. Since I only want this struct to represent a record in the table I decided to provide it with no logic and therefore it does not have a <code>impl</code> section. There are three Diesel specific attributes here: <code>#[derive(Queryable)]</code>, <code>#[derive(AsChangeSet)]</code> and <code>#[table_name]</code>. <code>#[derive(Queryable)]</code> will generate the code to retrieve a person from the database and <code>#[derive(AsChangeSet)]</code> to allow us to use <code>update.set</code> later on. Finally, <code>#[table_name = "people"]</code> is required since the plural of person in not people. If this struct was called post and the table posts, like in the Diesel <a href="http://diesel.rs/guides/getting-started/" target="_blank" rel="noopener">getting started example</a>, the attribute can be removed since the plural of post is posts which matches the table name.

The other attributes are aptly named; <code>#[derive(Serialize)]</code> and <code>#[derive(Deserialize)]</code>. These are for accepting/returning JSON into/from the REST API. They both come from the <code>serde</code> crate. We will look at this more later on in the post.

Before we move any further, we should look at creating our schema. Not a database schema for Postgres, a Rust schema file that uses the <code>table!</code> macro that does the actual Rust to database mappings for us. If we run the following command:
<pre>
diesel print-schema > src/schema.rs
</pre>
The following file is generated:
<pre>
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
</pre>
For now we can just ignore this file and carry on going.

Using the <code>Person</code> struct defined above, we can execute <code>SELECT</code> and <code>UPDATE</code> queries. <code>DELETE</code> doesn't require a struct to map to since we just require the record's ID. Then what about <code>INSERT</code>? For convenience, Diesel suggests doing it this way, we will use another struct with the sole purpose of being used for inserts.

[gist https://gist.github.com/lankydan/99adc54f48c4cf066446a1fd73e9d11f /]

<code>InsertablePerson</code> is nearly identical to the <code>Person</code> struct but with one difference, the <code>id</code> field is missing. This is because the ID of the record will be generated automatically when inserted, so we have no need to set it ourselves. Other fields could also differ slightly, if we don't want some other fields being set on creation. Similar to the <code>Person</code>'s attributes <code>#[derive(Insertable)]</code> is added generate the code to insert a new record.

I have also included an utility function <code>from_person</code> which takes a <code>Person</code> struct's values and converts it into an <code>InsertablePerson</code>. This simply removes the <code>id</code> field in this scenario and allows me to have tidier code in other places. This function isn't 100% necessary and is added due to my coding preferences.
<h3>Executing queries</h3>
At this point we have created our table and the structs that map to it. Now we need to put them to use. Below are all the methods needed to implement the basic REST API:

[gist https://gist.github.com/lankydan/bbf521dfb3d5f5380326274a51db49e3 /]

The <code>diesel</code> module is used to access the <code>insert_into</code>, <code>update</code> and <code>delete</code> functions. <code>diesel::prelude::*</code> provides access to a range of modules and structs that are generally useful when using Diesel, for this example; <code>PgConnection</code> and <code>QueryResult</code> are included in this list. <code>schema::people</code> is included so we can access the people table from within Rust and execute methods on it. Note that <code>schema::people</code> is referring back to the people table defined in the <code>schema.rs</code> file we generated earlier.

Let's look at one of the functions more closely:

[gist https://gist.github.com/lankydan/84ccaf17a2bb8db3fb5abb9acdc0a2d1 /]

As mentioned above, we can access the people table via <code>people::table</code> thanks to including <code>schema::people</code>. This example is nice and easy, <code>find</code> is specified as the query that selects a single record with the provided ID and <code>get_result</code> executes the query with the connection provided to it. 

In my examples <code>QueryResult</code> is returned from all functions. Diesel returns <code>QueryResult&lt;T&gt;</code> from most methods and is shorthand for <code>Result<T, Error></code> due to the following line:

[gist https://gist.github.com/lankydan/702de21e950da0c0711040b706fd869f /]

Returning <code>QueryResult</code> allows us to determine what happens if the query fails in whatever way is suitable for where the function is used. If we wanted to return a <code>Person</code> directly out of the function we could call <code>expect</code> to log the error there and then.

Also, since I have used Postgres for this post, <code>PgConnection</code> is used. If we were using one of the other databases Diesel support; MySql for example, <code>MysqlConnection</code> would be used instead.

Let's look at another one:

[gist https://gist.github.com/lankydan/2a530bf568823ce7724ba3721e1a58d2 /]

This works slightly differently to the earlier <code>get</code> function. Rather than accessing a function on the <code>people::table</code> it is passed into another Diesel function, <code>insert_into</code>. As I mentioned earlier in the post, <code>InsertablePerson</code> was defined specifically for new records, therefore the values from <code>person</code> are extracted thanks to the <code>from_person</code> helper function. Remember that no ID is included on this struct. Like before, <code>get_result</code> is called again to execute the statement.
<h2>Connection pooling - A bit of everything</h2>
You might have a question following on from the previous section. I'm hoping it's the question I'm about to answer... Where did the <code>PgConnection</code> come from? Well, let's have a look.

The code below shows how a connection pool is created:

[gist https://gist.github.com/lankydan/d17aff3353380e3f3992f105e91b40b2 /]

Now, I'm not going to lie. This is a straight up copy from the <a href="https://rocket.rs/guide/state/#databases" target="_blank" rel="noopener">Rocket documentation</a>. That link will probably provide a better explanation than I would but I'll give you a quick run through it.  <code>init_pool</code> creates a new pool of connections for our database which we have specified as <code>PgConnection</code>s. <code>DbConn</code> wraps the actual <code>PgConnection</code>. Finally, <code>FromRequest</code> allows a <code>DbConn</code> to be retrieved from Rocket handler functions when included in the input parameters, we will look at an example of this soon.
<h2>Rocket</h2>
All of the database magic has been implemented at this point. All we now need to do is create the REST API and hook it up to the back-end that we've created. In Rocket this consists of routes that map incoming requests to handler functions which will then deal with the requests. So we have got two clear things still to do, define the routes and create the handler functions.
<h3>Handlers</h3>
It makes sense to start with the handlers first so we actually have an idea of what the routes are mapping to. Below are all the handlers that are needed to implement the typical REST verbs of <code>GET</code>, <code>POST</code>, <code>PUT</code>, <code>DELETE</code>:

[gist https://gist.github.com/lankydan/c3e9fe301d87c270ba947ad9dbc3d130 /]

Each method is marked with an attribute that specifies what REST verb it accepts along with the path needed to get there. Part of the path is missing as the rest will be defined when the routes are created, so just hold on for a bit... The attributes can also accept a few extra properties to properly specify the behavior of the handler. 

Until we look at routing, just assume the base path to these handler methods are <code>localhost:8000/people</code>.

Let's look at one of the simpler handlers:

[gist https://gist.github.com/lankydan/c5e8a203cad89fad50d686c072a7a1b8 /]

This function returns all the person records stored in the database. It accepts a <code>GET</code> request thanks to the <code>#[get("/")]</code> attribute on the function. The path it accepts requests from is <code>localhost:8000/people</code> as denoted by the <code>"/"</code>.

To use cURL to send a request to this function we need to execute:
<pre>
curl localhost:8000/people
</pre>
This will then return a JSON list of people as specified by the return type of <code>Result&lt;Json&lt;Vec&lt;Person&gt;&gt;, Failure&gt;</code>. To do this, records are retrieved from database and mapped into their JSON representation. Thanks to the return type of <code>QueryResult&lt;Vec&lt;Person&gt;&gt;</code> from the <code>all</code> function, if anything goes wrong at the database level we can then map this to a HTTP status code to represent the error properly. This is why the return type is a <code>Result</code>. It provides us with an option to either return the records when nothing goes wrong but if anything does, it can return a error status code instead.

Serde finally pops up here, although it does so behind the scenes. Without the <code>#[derive(Serialize)]</code> attribute we added onto the <code>Person</code> struct earlier we would not be able to return <code>Json&lt;Vec&lt;Person&gt;&gt;</code> from this function; the same applies for <code>Json&lt;Person&gt;</code>.

The <code>error_status</code> function isn't particularly interesting and doesn't help for this specific example. It simply converts an <code>Error</code> contained within <code>QueryResult</code> into a status code. I was only particularly interested in these two scenarios, hence why it either returns <code>NotFound</code> or <code>InternalServerError</code> for anything else since I'm lazy (plus most of the other errors would honestly be classed as internal server errors).

The last point to touch on before we look at another handler function, the appearance of <code>DbConn</code>. The code we wrote earlier for connection pooling allows this. At this point all we need to do is include it in the function parameters and it will retrieve a connection for us.

Let's look at the <code>PUT</code> handler next:

[gist https://gist.github.com/lankydan/344731e805bd7a9982648fe4f185c0cd /]

The first difference between this function and the previous <code>ALL</code> example (ignoring request type) is the <code>id</code> and <code>person</code> being passed in. The <code>"&lt;/id&gt;"</code> represents the path variable <code>id</code> and <code>data = "&lt;person"&gt;</code> represents that request body that maps to <code>person</code> in the functions arguments. The <code>format</code> property specifies the content of the request body, in other words, the <code>data</code> property should contain JSON (indicated by <code>application/json</code>). We can see that it does indeed do just that since <code>person</code> is of type <code>Json&lt;Person&gt;</code>.

Serde again shows up here. It is needed to retrieve the <code>Json&lt;Person&gt;</code> from the request body.

To retrieve the contents of <code>person</code> we must call <code>into_inner()</code>, revealing the <code>Person</code> that was waiting to break out all along... <code>update</code> is called and the result or error is mapped and returned in the <code>Result</code> enum. Due to the implementation of <code>error_status</code>, an error will be thrown if an existing record does not exist with the passed in ID. Whether this is how it should work seems to vary from person to person (according to my googling anyway). If we instead wanted to insert the record if it did not already exist, we would need to handle the <code>Error::NotFound</code> and instead call similar code to that in the <code>POST</code> function.

Well we just mentioned it, so we need to look at it now. Below is the <code>POST</code> function:

[gist https://gist.github.com/lankydan/4c873148524fa652d7b99f660d874f07 /]

This contains similar components to the <code>PUT</code> function we just looked at. The main difference is the return type. The status code that should be returned from a successful <code>POST</code> request is <code>201 Created</code> rather than <code>200 Ok</code> which was used by the previous functions that we looked at. To return a different status code, the <code>Result</code> should contain <code>status::Created</code> instead of <code>Json&lt;Person&gt;</code> directly. This change is what makes it return a <code>201</code> status code.  

To create the <code>status::Created</code> struct, the created record along with the path to retrieve it (via a <code>GET</code> request) must be passed into it's constructor. Passing in the path as an absolute string isn't ideal so I have retrieved the host and port number from the environment variables. This might not be the best way to get this to work... But I spent ages trying to figure out how to get them out of Rocket and gave up in the end.

We should probably also look at Responders in Rocket and how they enrich the returned responses, but this post is already so long so I will instead refer you to the <a href="https://rocket.rs/guide/responses/#rocket-responders" target="_blank" rel="noopener">Rocket documentation</a> on the subject.
<h3>Routing</h3>
We are nearly at the end now... Don't give up yet!

The handlers are setup to accept requests to the server but before we can use them the we need to set the routes to the different functions. Since all of the functions in this post are related to people it will be mapped to <code>/people</code>. See the code below on how to do this:

[gist https://gist.github.com/lankydan/426ede1ffedafe52aa965f6a00ca9120 /]

<code>create_routes</code> is called by the <code>main</code> function to get everything rolling. <code>ignite</code> creates a new instance of <code>Rocket</code>. The handler functions are then mounted onto a base request path of <code>/people</code> by specifying all of the them inside of <code>routes!</code>. Finally, <code>launch</code> starts the application server.
<h3>Configuration</h3>
Earlier in this post I made use of environment variables to retrieve the host and port of the running server. So let's have a brief look at the configuration required to change the host and port in Rocket. There are two ways to do this from within configuration files. Either specify values within a <code>.env</code> file or create a <code>Rocket.toml</code> file. 

When using a <code>.env</code> file, the values must follow the format of <code>ROCKET_{PARAM}</code> where <code>PARAM</code> is the property you are trying to set. <code>{ADDRESS}</code> represents the host and <code>{PORT}</code> is obviously the port number. Taking this information, below is the <code>.env</code> file used in this post (removing unrelated configuration):
<pre>
ROCKET_ADDRESS=localhost
ROCKET_PORT=8000
</pre>
If instead you wanted to use a <code>Rocket.toml</code> file, it would look like the below.
<pre>
[development]
address = "localhost"
port = 8000
</pre>
In this situation, these values are only applicable for development, which is handy since thats all I'm doing.

If you choose to include neither of these Rocket will instead fall back to it's default configuration. So don't worry about needing to do loads of configuration when playing around with Rocket; for local development the defaults are most likely good enough.

For more (and better) explanations of Rocket Configuration, I again recommend looking at their <a href="https://rocket.rs/guide/configuration/#configuration" target="_blank" rel="noopener">documentation</a>.
<h2>The last step</h2>
Finally we have reached the end. All that is left to do now is create the <code>main</code> method so the application can be run.

[gist https://gist.github.com/lankydan/3efb9383579a23a1d40d502a2c450068 /]

All <code>main</code> does is load in the environment variables and starts Rocket by calling <code>create_routes</code>. The rest of this file just pulls in a load of crates so they don't need to be scattered throughout the rest of the code.

Now you can rest. That was a pretty long post. I'd write a conclusion but honestly, I'm tired and don't want to write anymore. So for a short summary, in this post we have created a simple REST API using Rocket to run an application server that responds to requests and used Diesel to connect to a database to manage the state of the application.

The code used in this post can be on my <a href="https://github.com/lankydan/rust-web-with-rocket" target="_blank" rel="noopener">GitHub</a>.

If you liked this post, then follow me on Twitter at <a href="https://twitter.com/LankyDanDev" target="_blank" rel="noopener">@LankyDanDev</a> to be able to keep up with my new posts as I write them.

