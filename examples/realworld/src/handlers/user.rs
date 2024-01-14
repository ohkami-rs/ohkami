use ohkami::{Ohkami, Route, utils::Payload, typed::OK, Memory};
use crate::{
    fangs::Auth,
    models::{User, UserResponse},
    errors::RealWorldError,
    config::{pool, issue_jwt_for_user_of_id, JWTPayload},
    db::{UserEntity, hash_password_string},
};


pub fn user_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/"
            .GET(get_current_user)
            .POST(update),
    ))
}

async fn get_current_user(
    jwt_payload: Memory<'_, JWTPayload>
) -> Result<OK<UserResponse>, RealWorldError> {
    let u = sqlx::query_as!(UserEntity, r#"
        SELECT id, email, name, bio, image_url
        FROM users AS u
        WHERE
            u.id = $1
    "#, jwt_payload.user_id)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(OK(UserResponse {
        user: User {
            email: u.email,
            jwt:   issue_jwt_for_user_of_id(u.id),
            name:  u.name,
            bio:   u.bio,
            image: u.image_url,
        },
    }))
}

#[Payload(JSOND)]
struct UpdateRequest {
    email:    Option<String>,
    username: Option<String>,
    password: Option<String>,
    image:    Option<String>,
    bio:      Option<String>,
}

async fn update(
    body:        UpdateRequest,
    jwt_payload: Memory<'_, JWTPayload>,
) -> Result<OK<UserResponse>, RealWorldError> {
    let user_entity = {
        let UpdateRequest { email, username, image, bio, password:raw_password } = body;
        let password = raw_password.map(hash_password_string).transpose()?;

        let mut set_once = false;
        macro_rules! set_if_some {
            ($field:ident -> $query:ident . $column:ident) => {
                if let Some($field) = $field {
                    if set_once {$query.push(',');}
                    $query.push(concat!(" ",stringify!($column)," = ")).push_bind($field);
                    set_once = true; 
                }
            };
        }

        let mut query = sqlx::QueryBuilder::new("UPDATE users SET");
        set_if_some!(email    -> query.email);
        set_if_some!(username -> query.name);
        set_if_some!(password -> query.password);
        set_if_some!(image    -> query.image_url);
        set_if_some!(bio      -> query.bio);
        query.push(" WHERE id = ").push_bind(jwt_payload.user_id);
        query.push(" RETURNING id, email, name, image_url, bio");

        if !set_once {
            // Requested to update nothing, then
            // not perform UPDATE query
            return get_current_user(jwt_payload).await
        }

        query.build_query_as::<UserEntity>()
            .fetch_one(pool()).await
            .map_err(RealWorldError::DB)?
    };

    Ok(OK(user_entity.into_user_response()))
}
