use crate::{config, errors::RealWorldError};
use crate::models::{User, UserResponse, ProfileResponse, Profile};
use argon2::{Argon2, Algorithm, Version, Params, PasswordHasher};
use argon2::password_hash::{PasswordHashString, Salt, SaltString};
use chrono::{DateTime, Utc};
use uuid::Uuid;


pub fn hash_password(
    raw_password: &str,
) -> Result<PasswordHashString, RealWorldError> {
    let a = Argon2::new_with_secret(
        config::pepper(),
        Algorithm::Argon2id,
        Version::V0x13,
        Params::DEFAULT,
    ).map_err(|e| RealWorldError::Config(e.to_string()))?;

    let s = SaltString::generate(::argon2::password_hash::rand_core::OsRng);

    let hash = a.hash_password(
        raw_password.as_bytes(),
        Salt::from(&s),
    ).map_err(|e| RealWorldError::Config(e.to_string()))?;

    Ok(hash.serialize())
}

#[inline] pub fn hash_password_string(
    raw_password_string: String,
) -> Result<String, RealWorldError> {
    let hashed_password = hash_password(raw_password_string.as_str())?;
    Ok(hashed_password.as_str().to_string())
}

#[derive(sqlx::FromRow)]
pub struct UserEntity {
    pub id:        Uuid,
    pub email:     String,
    pub name:      String,
    pub bio:       Option<String>,
    pub image_url: Option<String>,
} impl UserEntity {
    pub fn into_user_response(self) -> UserResponse {
        UserResponse {
            user: User {
                email: self.email,
                jwt:   config::issue_jwt_for_user_of_id(self.id),
                name:  self.name,
                bio:   self.bio,
                image: self.image_url,
            },
        }
    }
    pub fn into_profile_response_with(self, following: bool) -> ProfileResponse {
        ProfileResponse {
            profile: Profile {
                username: self.name,
                bio:      self.bio,
                image:    self.image_url,
                following
            },
        }
    }
} impl UserEntity {
    pub async fn get_by_name(name: &str) -> Result<Self, RealWorldError> {
        sqlx::query_as!(UserEntity, r#"
            SELECT id, name, email, bio, image_url
            FROM users AS u
            WHERE
                u.name = $1
        "#, name)
            .fetch_one(config::pool()).await
            .map_err(RealWorldError::DB)
    }
}

#[derive(sqlx::FromRow)]
pub struct ArticleEntity {
    pub slug:        String,
    pub title:       String,
    pub description: String,
    pub body:        String,
    pub created_at:  DateTime<Utc>,
    pub updated_at:  DateTime<Utc>,
    
}
/*
pub slug:           String,
pub title:          String,
pub description:    String,
pub body:           String,
#[serde(rename = "tagList")]
pub tag_list:       Vec<String>,
#[serde(rename = "createdAt", serialize_with = "serialize_datetime")]
pub created_at:     DateTime<Utc>,
#[serde(rename = "updatedAt", serialize_with = "serialize_datetime")]
pub updated_at:     DateTime<Utc>,
pub favorited:      bool,
#[serde(rename = "favoriteCount")]
pub favorite_count: usize,
pub author:         Profile,

*/
