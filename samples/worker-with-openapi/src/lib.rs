mod error;
mod model;

use error::APIError;

use ohkami::prelude::*;
use ohkami::fang::BasicAuth;
use ohkami::typed::status;
use ohkami::format::JSON;

#[cfg(feature="openapi")]
use ohkami::openapi::{self, operation, Schema};

#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
pub fn ohkami() -> Ohkami {
    /* `pages` directory is served via `--assets pages` flag (see package.json) */

    let openapi_doc_server_ohkami = Ohkami::with(BasicAuth {
        username: "ohkami",
        password: Bindings::OPENAPI_DOC_PASSWORD
    }, (
        "/".GET(|| async {include_str!("../openapi.json")})
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
struct UserProfile {
    name:    String,
    country: Option<String>,
    age:     Option<u8>,
}

async fn list_users(
    Bindings { DB, .. }: Bindings
) -> Result<JSON<Vec<UserProfile>>, APIError> {
    let users = DB.prepare("SELECT name, country, age FROM users ORDER BY id")
        .all().await?
        .results::<UserProfile>()?;

    Ok(JSON(users))
}

#[derive(Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
struct SignUpRequest<'req> {
    name: &'req str,
    /// just for demo
    token: &'req str,
}

async fn sign_up(
    JSON(req): JSON<SignUpRequest<'_>>,
    Bindings { DB, .. }: Bindings,
) -> Result<status::Created<JSON<UserProfile>>, APIError> {
    let already_used = DB.prepare("SELECT EXISTS (SELECT id FROM users WHERE name = ?)")
        .bind(&[req.name.into()])?
        .first::<u8>(Some("exists")).await?;

    (already_used != Some(1)).then_some(()).ok_or_else(||
        APIError::UserNameAlreadyUsed(req.name.into())
    )?;

    let id = DB.prepare("INSERT INTO users (name, token) VALUES (?, ?)")
        .bind(&[req.name.into(), req.token.into()])?
        .run().await?;
    
    Ok(status::Created(JSON(UserProfile {
        name:    req.name.into(),
        country: None,
        age:     None,
    })))
}

struct Token<'req>(&'req str);
impl<'req> ohkami::FromRequest<'req> for Token<'req> {
    type Error = std::convert::Infallible;
    
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        let token = req.headers
            .Authorization()?
            .strip_prefix("Bearer ")?;
        Some(Ok(Self(token)))
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Security {
            scheme: openapi::SecurityScheme::Bearer("tokenAuth", None),
            scopes: &[]
        }
    }
}

struct Tweet {
    content:   String,
    posted_at: u64,
}

async fn post_tweet(
    Token(token): Token<'_>,
    Bindings { DB, .. }: Bindings,
) -> Result<>
