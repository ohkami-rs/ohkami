use std::borrow::Cow;
use argon2::PasswordHasher;
use ohkami::{Ohkami, Route, utils::{Payload, Deserialize, Deserializer, base64}, typed::{Created, OK}};
use ohkami::utils::base64::encode;
use uuid::Uuid;
use crate::{models::{User, UserResponse}, errors::RealWorldError, config::{pool, self}};


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
} const _: () = {
    impl<'req> Deserialize<'req> for LoginRequest<'req> {
        fn deserialize<D: Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Self {
                user: LoginRequestUser::deserialize(deserializer)?,
            })
        }
    }
};
#[derive(Deserialize)]
struct LoginRequestUser<'req> {
    email:    &'req str,
    password: &'req str,
}

async fn login(body: LoginRequest<'_>) -> Result<OK<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSOND)]
struct RegisterRequest<'req> {
    username: &'req str,
    email:    &'req str,
    password: &'req str,
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest<'_>,
) -> Result<Created<UserResponse>, RealWorldError> {
    let already_exists = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users AS u
            WHERE
                u.name  = $1 AND
                u.email = $2
        )
    "#, username, email).fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();
    if already_exists {
        return Err(RealWorldError::FoundUnexpectedly(Cow::Owned(
            format!("User of name = '{username}' & email = '{email}' is already exists")
        )))
    }

    let new_user_id = Uuid::new_v4();

    let hashed_password = {
        use ::argon2::{Argon2, Algorithm, Version, Params, password_hash::Salt};

        let a = Argon2::new_with_secret(
            config::pepper(),
            Algorithm::Argon2id,
            Version::V0x13,
            Params::DEFAULT,
        ).map_err(|e| RealWorldError::Config(e.to_string()))?;

        let salt = base64::encode(&new_user_id);

        let hash = a.hash_password(
            password.as_bytes(),
            Salt::from_b64(&salt).map_err(|e| RealWorldError::Config(e.to_string()))?,
        ).map_err(|e| RealWorldError::Config(e.to_string()))?;

        hash.serialize()
    };

    sqlx::query!(r#"
        INSERT INTO
            users  (email, name, password)
            VALUES ($1,    $2,   $3)
    "#, email, username, hashed_password.as_str()).execute(pool()).await
        .map_err(RealWorldError::DB)?;

    let jwt_token = config::issue_jwt_for_user_of_id(new_user_id);

    Ok(Created(UserResponse {
        user: User {
            email:    email.into(),
            token:    jwt_token,
            username: username.into(),
            bio:      None,
            image:    None,
        },
    }))
}
