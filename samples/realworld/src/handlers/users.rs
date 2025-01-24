use ohkami::prelude::*;
use ohkami::typed::status::Created;
use sqlx::PgPool;
use crate::{
    models::User,
    models::response::UserResponse,
    models::request::{LoginRequest, LoginRequestUser, RegisterRequest, RegisterRequestUser},
    errors::RealWorldError,
    config,
    db,
};


pub fn users_ohkami() -> Ohkami {
    Ohkami::new((
        "/login"
            .POST(login),
        "/"
            .POST(register),
    ))
}

async fn login(
    Context(pool): Context<'_, PgPool>,
    JSON(LoginRequest {
        user: LoginRequestUser { email, password },
    }): JSON<LoginRequest<'_>>,
) -> Result<JSON<UserResponse>, RealWorldError> {
    let credential = sqlx::query!(r#"
        SELECT password, salt
        FROM users
        WHERE email = $1
    "#, email)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    db::verify_password(password, &credential.salt, &credential.password)?;

    let u = sqlx::query_as!(db::UserEntity, r#"
        SELECT id, email, name, bio, image_url
        FROM users AS u
        WHERE email = $1
    "#, email)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(JSON(u.into_user_response()?))
}

async fn register(
    Context(pool): Context<'_, PgPool>,
    JSON(RegisterRequest {
        user: RegisterRequestUser { username, email, password }
    }): JSON<RegisterRequest<'_>>,
) -> Result<Created<JSON<UserResponse>>, RealWorldError> {
    let already_exists = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users AS u
            WHERE
                u.name = $1
        )
    "#, username)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();
    if already_exists {
        return Err(RealWorldError::Validation {
            body: format!("User of name {username:?} is already exists")
        })
    }

    let (hased_password, salt) = db::hash_password(password)?;

    let new_user_id = sqlx::query!(r#"
        INSERT INTO
            users  (email, name, password, salt)
            VALUES ($1,    $2,   $3,       $4  )
        RETURNING id
    "#, email, username, hased_password.as_str(), salt.as_str())
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?
        .id;

    Ok(Created(JSON(
        UserResponse {
            user: User {
                email: email.into(),
                jwt:   config::issue_jwt_for_user_of_id(new_user_id)?,
                name:  username.into(),
                bio:   None,
                image: None,
            },
        }
    )))
}
