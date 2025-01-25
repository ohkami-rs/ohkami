use ohkami::prelude::*;
use sqlx::PgPool;
use crate::{
    fangs::Auth,
    models::User,
    models::response::UserResponse,
    models::request::{UpdateProfileRequest, UpdateProfileRequestUser},
    errors::RealWorldError,
    config::JWTPayload,
    db::{UserEntity, hash_password},
};


pub fn user_ohkami() -> Ohkami {
    Ohkami::new((
        Auth::required(),
        "/"
            .GET(get_user)
            .PUT(update),
    ))
}

async fn get_user(
    Context(pool): Context<'_, PgPool>,
    Context(auth): Context<'_, JWTPayload>,
) -> Result<JSON<UserResponse>, RealWorldError> {
    let user = util::get_current_user(pool, auth).await?;
    Ok(JSON(UserResponse { user }))
}

async fn update(
    JSON(req): JSON<UpdateProfileRequest<'_>>,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<UserResponse>, RealWorldError> {
    let user_entity = {
        let UpdateProfileRequest {
            user: UpdateProfileRequestUser {
                email, username, image, bio, password:raw_password
            }
        } = req;
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
            // Requested to update nothing, then not perform UPDATE query
            let user = util::get_current_user(pool, auth).await?;
            return Ok(JSON(UserResponse { user }))
        }

        query.build_query_as::<UserEntity>()
            .fetch_one(pool).await
            .map_err(RealWorldError::DB)?
    };

    Ok(JSON(user_entity.into_user_response()?))
}

mod util {
    use super::*;

    pub async fn get_current_user<'a>(
        pool: &'a PgPool,
        auth: &'a JWTPayload,
    ) -> Result<User, RealWorldError> {
        let u = sqlx::query_as!(UserEntity, r#"
            SELECT id, email, name, bio, image_url
            FROM users AS u
            WHERE
                u.id = $1
        "#, auth.user_id)
            .fetch_one(pool).await
            .map_err(RealWorldError::DB)?;

        Ok(u.into_user()?)
    }
}
