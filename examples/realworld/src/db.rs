use std::borrow::Cow;
use std::str::FromStr;

use crate::{config, errors::RealWorldError};
use crate::models::{User, UserResponse, ProfileResponse, Profile, Article};
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
    pub id:              Uuid,
    pub slug:            Option<String>,
    pub title:           String,
    pub description:     Option<String>,
    pub body:            String,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    pub favorites_count: Option<i64>,
    pub favoriter_ids:   Option<Vec<Uuid>>,
    pub tags:            Option<Vec<String>>,
    pub authors:         Option<sqlx::types::JsonValue>,
} impl ArticleEntity {
    pub fn into_article_with(
        self,
        user_and_followings: Option<(Uuid, Cow<'static, [Uuid]>)>,
    ) -> Article {
        let user_favorited_this_article = match &user_and_followings {
            None => false,
            Some((id, _)) => self.favoriter_ids.unwrap_or_else(Vec::new).contains(id),
        };

        let author_profile = match self.authors.expect("No author found") {
            sqlx::types::JsonValue::Array(mut arr) => {
                assert!(arr.len() == 1, "Multiple authors found; this may be allowed in future, but not implemented now");
                let mut author_obj = match unsafe {arr.pop().unwrap_unchecked()} {
                    sqlx::types::JsonValue::Object(obj) => obj,
                    other => unreachable!("author was not object: {other:?}")
                };

                let id = match author_obj.remove("id").expect("`id` was not found in author") {
                    sqlx::types::JsonValue::String(s) => Uuid::from_str(&s).unwrap(),
                    other => unreachable!("`id` was not string in `author_obj`: {other:?}"),
                };
                let username = match author_obj.remove("name").expect("`name` was not found in author") {
                    sqlx::types::JsonValue::String(s) => s,
                    other => unreachable!("`name` was not string: {other:?}")
                };
                let bio = match author_obj.remove("bio") {
                    Some(sqlx::types::JsonValue::String(s))   => Some(s),
                    None | Some(sqlx::types::JsonValue::Null) => None,
                    Some(other) => unreachable!("`bio` was not string: {other:?}")
                };
                let image = match author_obj.remove("image_url") {
                    Some(sqlx::types::JsonValue::String(s))   => Some(s),
                    None | Some(sqlx::types::JsonValue::Null) => None,
                    Some(other) => unreachable!("`image_url` was not string: {other:?}")
                };

                Profile {
                    username,
                    bio,
                    image,
                    following: user_and_followings
                        .map(|(_, following_authors)| following_authors.contains(&id))
                        .unwrap_or(false),
                }
            }
            other => unreachable!("`authors` was not json array: {other:?}")
        };

        Article {
            title:           self.title,
            slug:            self.slug,
            description:     self.description,
            body:            self.body,
            tag_list:        self.tags.unwrap_or_else(Vec::new),
            created_at:      self.created_at,
            updated_at:      self.updated_at,
            favorites_count: self.favorites_count.unwrap_or(0) as _,
            favorited:       user_favorited_this_article,
            author:          author_profile,
        }
    }
}
