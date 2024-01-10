use ohkami::{Ohkami, Route, utils::{Payload, JSON}};
use serde::Deserialize;
use crate::{models::{User, UserResponse}, errors::RealWorldError, config::pool};


pub fn users_ohkami() -> Ohkami {
    Ohkami::new((
        "/login"
            .POST(login),
        "/"
            .POST(register),
    ))
}

#[Payload(JSON)]
struct LoginRequest<'req> {
    user: LoginRequestUser<'req>,
}
impl<'req> Deserialize<'req> for LoginRequest<'req> {
    fn deserialize<D: serde::Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            user: LoginRequestUser::deserialize(deserializer)?,
        })
    }
}
#[derive(Deserialize)]
struct LoginRequestUser<'req> {
    email:    &'req str,
    password: &'req str,
}

async fn login(body: LoginRequest<'_>) -> Result<JSON<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest<'req> {
    username: &'req str,
    email:    &'req str,
    password: &'req str,
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest<'_>,
    //pool: ohkami::Memory<'_, sqlx::PgPool>,
) -> Result<JSON<UserResponse>, RealWorldError> {
    let _ = sqlx::query("").execute(pool()).await;
    // sqlx::query!(r#"
    //     SELECT id
    //     FROM users AS u
    //     WHERE
    //         u.name = $1  AND
    //         u.email = $2
    //         -- TODO
    // "#, username, email).fetch_optional(pool()).await.map_err(RealWorldError::DB)?;

    Ok(JSON::Created(UserResponse {
        user: User {
            email:    String::new(),
            token:    String::new(),
            username: String::new(),
            bio:      None,
            image:    None,
        }
    }))
}

#[cfg(test)] fn __() {
    fn assert_fn1<
        'req,
        F:   Fn(B) -> Fut + Send + Sync + 'static,
        B:   ohkami::FromRequest<'req>,
        Fut: std::future::Future<Output = Res> + 'static,
        Res: ohkami::IntoResponse,
    >(_: F) {}
    fn assert_fn2<
        'req,
        F:   Fn(B1, B2) -> Fut + Send + Sync + 'static,
        B1:  ohkami::FromRequest<'req>,
        B2:  ohkami::FromRequest<'req>,
        Fut: std::future::Future<Output = Res> + 'static,
        Res: ohkami::IntoResponse,
    >(_: F) {}

    assert_fn1(login);
    assert_fn1(register);
}
