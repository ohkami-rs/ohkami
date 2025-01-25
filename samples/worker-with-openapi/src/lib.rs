mod error;
mod fang;
mod model;

use error::APIError;
use fang::{TokenAuth, TokenAuthed, Logger};
use model::*;

use ohkami::prelude::*;
use ohkami::fang::BasicAuth;
use ohkami::typed::status;
use ohkami::format::JSON;

#[ohkami::bindings]
struct Bindings;

#[ohkami::worker]
pub fn ohkami() -> Ohkami {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let openapi_doc_server_ohkami = Ohkami::new((
        BasicAuth {
            username: "ohkami",
            password: Bindings::OPENAPI_DOC_PASSWORD
        },
        "/".GET(|| async {include_str!("../openapi.json")})
    ));

    let api_ohkami = Ohkami::new((
        "/users"
            .GET(list_users)
            .POST(sign_up),
        "/users/:id"
            .GET(show_user_profile)
            .PUT((TokenAuth, edit_profile)),
        "/tweets"
            .GET(list_tweets)
            .POST((TokenAuth, post_tweet)),
    ));

    Ohkami::new((
        Logger,
        "/openapi.json".By(openapi_doc_server_ohkami),
        "/api".By(api_ohkami)
    ))
}

async fn show_user_profile(id: ID,
    Bindings { DB, .. }: Bindings,
) -> Result<JSON<UserProfile>, APIError> {
    let user_proifle = DB.prepare("SELECT id, name, location, age FROM users WHERE id = ?")
        .bind(&[id.into()])?
        .first::<UserProfile>(None).await?
        .ok_or(APIError::UserNotFound { id })?;
    
    Ok(JSON(user_proifle))
}

async fn edit_profile(id: ID,
    JSON(req): JSON<EditProfileRequest<'_>>,
    Context(TokenAuthed { user_id, .. }): Context<'_, TokenAuthed>,
    Bindings { DB, .. }: Bindings,
) -> Result<(), APIError> {
    if *user_id != id {
        return Err(APIError::ModifyingOtherUser { me: *user_id, other: id });
    }

    macro_rules! make_set_clause {
        ($( $field:ident ),* ) => {{
            let mut clause = String::from("SET ");
            let mut params = Vec::<worker::wasm_bindgen::JsValue>::new();
            $(
                if let Some($field) = req.$field {
                    if !params.is_empty() {
                        clause.push(',');
                    }
                    clause.push_str(concat!(stringify!($field), " = ?"));
                    params.push($field.into());
                }
            )*
            (!params.is_empty()).then_some((clause, params))
        }}
    }

    if let Some((set_clause, mut params)) = make_set_clause!(location, age) {
        params.push((*user_id).into());
        DB.prepare(["UPDATE users ", &set_clause, " WHERE id = ?"].concat())
            .bind(&params)?
            .run().await?;
    }

    Ok(())
}

async fn list_users(
    Bindings { DB, .. }: Bindings,
) -> Result<JSON<Vec<UserProfile>>, APIError> {
    let users = DB.prepare("SELECT id, name, location, age FROM users ORDER BY id")
        .all().await?
        .results::<UserProfile>()?;

    Ok(JSON(users))
}

async fn sign_up(
    JSON(req): JSON<SignUpRequest<'_>>,
    Bindings { DB, .. }: Bindings,
) -> Result<status::Created<JSON<UserProfile>>, APIError> {
    let already_used = DB.prepare("SELECT EXISTS (SELECT id FROM users WHERE name = ?) as e")
        .bind(&[req.name.into()])?
        .first::<u8>(Some("e")).await?;

    (already_used != Some(1)).then_some(()).ok_or_else(||
        APIError::UserNameAlreadyUsed(req.name.into())
    )?;

    let id = DB.prepare("INSERT INTO users (name, token) VALUES (?, ?) RETURNING id")
        .bind(&[req.name.into(), req.token.into()])?
        .first::<ID>(Some("id")).await?
        .ok_or_else(|| APIError::Internal(format!(
            "Failed to insert user (name = `{}`, token = `{}`) and fetch id",
            req.name,
            req.token
        )))?;
    
    Ok(status::Created(JSON(UserProfile {
        id,
        name:     req.name.into(),
        location: None,
        age:      None,
    })))
}

async fn list_tweets(
    Bindings { DB, .. }: Bindings
) -> Result<JSON<Vec<Tweet>>, APIError> {
    let tweets = DB.prepare("\
        SELECT \
            t.user_id, \
            u.name AS user_name, \
            t.content, \
            t.posted_at \
        FROM \
            tweets AS t \
        JOIN \
            users AS u \
            ON t.user_id = u.id \
        ORDER BY \
            t.posted_at \
    ").all().await?.results::<Tweet>()?;

    Ok(JSON(tweets))
}

async fn post_tweet(
    JSON(req): JSON<PostTweetRequest<'_>>,
    Context(TokenAuthed { user_id, user_name }): Context<'_, TokenAuthed>,
    Bindings { DB, .. }: Bindings,
) -> Result<status::Created<JSON<Tweet>>, APIError> {
    let timestamp = crate::model::timestamp_now();

    DB.prepare("INSERT INTO tweets (user_id, content, posted_at) VALUES (?, ?, ?)")
        .bind(&[(*user_id).into(), req.content.into(), (&*timestamp).into()])?
        .run().await?;

    Ok(status::Created(JSON(Tweet {
        user_id:   *user_id,
        user_name: user_name.into(),
        content:   req.content.into(),
        posted_at: timestamp,
    })))
}
