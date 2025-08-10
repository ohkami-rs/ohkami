use ohkami::prelude::*;
use ohkami::claw::status::Created;
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
    Json(LoginRequest {
        user: LoginRequestUser { email, password },
    }): Json<LoginRequest<'_>>,
) -> Result<Json<UserResponse>, RealWorldError> {
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

    Ok(Json(u.into_user_response()?))
}

async fn register(
    Context(pool): Context<'_, PgPool>,
    Json(RegisterRequest {
        user: RegisterRequestUser { username, email, password }
    }): Json<RegisterRequest<'_>>,
) -> Result<Created<Json<UserResponse>>, RealWorldError> {
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

    Ok(Created(Json(
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
