use ohkami::{Ohkami, Route, Memory};
use sqlx::PgPool;
use crate::{
    fangs::Auth,
    models::User,
    models::response::UserResponse,
    models::request::{UpdateProfileRequest, UpdateProfileRequestUser},
    errors::RealWorldError,
    config::{issue_jwt_for_user_of_id, JWTPayload},
    db::{UserEntity, hash_password},
};


pub fn user_ohkami() -> Ohkami {
    Ohkami::with(Auth::default(), (
        "/"
            .GET(get_current_user)
            .PUT(update),
    ))
}

async fn get_current_user(
    pool: Memory<'_, PgPool>,
    auth: Memory<'_, JWTPayload>,
) -> Result<UserResponse, RealWorldError> {
    let u = sqlx::query_as!(UserEntity, r#"
        SELECT id, email, name, bio, image_url
        FROM users AS u
        WHERE
            u.id = $1
    "#, auth.user_id)
        .fetch_one(*pool).await
        .map_err(RealWorldError::DB)?;

    Ok(UserResponse {
        user: User {
            email: u.email,
            jwt:   issue_jwt_for_user_of_id(u.id)?,
            name:  u.name,
            bio:   u.bio,
            image: u.image_url,
        },
    })
}

async fn update<'h>(
    body: UpdateProfileRequest<'h>,
    auth: Memory<'h, JWTPayload>,
    pool: Memory<'h, PgPool>,
) -> Result<UserResponse, RealWorldError> {
    let user_entity = {
        let UpdateProfileRequest {
            user: UpdateProfileRequestUser {
                email, username, image, bio, password:raw_password
            }
        } = body;
        let new_password_and_salt = raw_password.map(|rp| hash_password(&rp)).transpose()?;

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
        set_if_some!(image    -> query.image_url);
        set_if_some!(bio      -> query.bio);
        if let Some((hash, salt)) = new_password_and_salt {
            if set_once {query.push(',');}
            query.push(" password = ").push_bind(hash.as_str().to_string());
            query.push(" salt = ").push_bind(salt.as_str().to_string());
        }
        query.push(" WHERE id = ").push_bind(auth.user_id);
        query.push(" RETURNING id, email, name, image_url, bio");

        if !set_once {
            // Requested to update nothing, then
            // not perform UPDATE query
            return get_current_user(pool, auth).await
        }

        query.build_query_as::<UserEntity>()
            .fetch_one(*pool).await
            .map_err(RealWorldError::DB)?
    };

    Ok(user_entity.into_user_response()?)
}
