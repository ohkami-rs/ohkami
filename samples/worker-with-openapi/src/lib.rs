mod error;
mod model;

use error::APIError;

use ohkami::prelude::*;
use ohkami::fang::BasicAuth;
use ohkami::typed::status;
use ohkami::format::JSON;
use ohkami::openapi::{operation, Schema};

#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
pub fn ohkami() -> Ohkami {
    /* `pages` directory is served via `--assets pages` flag (see package.json) */

    let openapi_doc_server_ohkami = Ohkami::with(BasicAuth {
        name: "ohkami",
        password: Bindings::OPENAPI_DOC_PASSWORD
    }, (
        "/".GET(|| async {include_str!("openapi.json")})
    ));

    let api_ohkami = Ohkami::new((
        "/users"
            .GET(list_users),
    ));

    Ohkami::new((
        "/openapi.json".By(openapi_doc_server_ohkami),
        "/api".By(api_ohkami)
    ))
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
struct UserResponse {
    id:      i32,
    name:    String,
    country: Option<String>,
    age:     Option<u8>,
}

async fn list_users(
    Bindings { DB, .. }: Bindings
) -> Result<JSON<Vec<UserResponse>>, APIError> {
    let users = DB.prepare("SELECT id, name, country, age FROM users ORDER BY id")
        .all().await?
        .results::<UserResponse>()?;

    Ok(JSON(users))
}

struct SignUpRequest<'req> {
    name: &'req str,
    /// just for demo
    token: &'req str,
}

async fn sign_up(
    JSON(req): JSON<SignUpRequest<'_>>,
    Bindings { DB, .. }: Bindings,
) -> Result<status::Created<JSON<UserResponse>>, APIError> {
    let already_used = DB.prepare("SELECT EXISTS (SELECT id FROM users WHERE name = ?)")
        .bind(&[req.name.into()])?
        .first::<u8>(Some("exists")).await?;

    (already_used != Some(1)).then_some(()).ok_or_else(||
        APIError::UserNameAlreadyUsed(req.name.into())
    )?;

    let id = DB.prepare("INSERT INTO users (name, token) VALUES (?, ?) RETURNING id")
        .bind(&[req.name.into(), req.token.into()])
        .first::<i32>(Some("id")).await?
        .unwrap();
    
    Ok(status::Created(JSON(UserResponse {
        id,
        name:    req.name.into(),
        country: None,
        age:     None,
    })))
}

async fn post_tweet(
    
)
