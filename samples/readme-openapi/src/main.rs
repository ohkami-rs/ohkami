use ohkami::prelude::*;
use ohkami::typed::status;
use ohkami::openapi;

// Derive `Schema` trait to generate
// the schema of this struct in OpenAPI document.
#[derive(Deserialize, openapi::Schema)]
struct CreateUser<'req> {
    name: &'req str,
}

#[derive(Serialize, openapi::Schema)]
// `#[openapi(component)]` to define it as component
// in OpenAPI document.
#[openapi(component)]
struct User {
    id: usize,
    name: String,
}

async fn create_user(
    JSON(CreateUser { name }): JSON<CreateUser<'_>>
) -> status::Created<JSON<User>> {
    status::Created(JSON(User {
        id: 42,
        name: name.to_string()
    }))
}

// (optionally) Set operationId, summary,
// or override descriptions by `operation` attribute.
#[openapi::operation({
    summary: "...",
    200: "List of all users",
})]
/// This doc comment is used for the
/// `description` field of OpenAPI document
async fn list_users() -> JSON<Vec<User>> {
    JSON(vec![])
}

#[tokio::main]
async fn main() {
    let o = Ohkami::new((
        "/users"
            .GET(list_users)
            .POST(create_user),
    ));

    // This make your Ohkami spit out `openapi.json`
    // ( the file name is configurable by `.generate_to` ).
    o.generate(openapi::OpenAPI {
        title: "Users Server",
        version: "0.1.0",
        servers: &[
            openapi::Server::at("localhost:5000"),
        ]
    });

    o.howl("localhost:5000").await;
}
