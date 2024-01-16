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
    pub id:              Uuid,
    pub slug:            String,
    pub title:           String,
    pub description:     String,
    pub body:            String,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    pub favorites_count: i64,
    pub author:          Vec<AuthorEntity>,
    pub tags:            Vec<String>,
    pub favoriter_ids:   Vec<Uuid>,
}
#[derive(sqlx::FromRow)]
pub struct AuthorEntity {
    pub name:      String,
    pub bio:       Option<String>,
    pub image_url: Option<String>,
}

/*
    let mut query = sqlx::QueryBuilder::new(sqlx::query!(r#"
        SELECT
            a.id                  AS article_id,
            a.slug                AS article_slug,
            a.title               AS article_title,
            a.description         AS article_description,
            a.body                AS article_body,
            a.created_at          AS article_created_at,
            a.updated_at          AS article_updated_at,
            COUNT(fav.id)         AS favorites_count,
            JSON_AGG(users)       AS author,
            JSON_AGG(tags.name)   AS tags,
            JSON_AGG(fav.user_id) AS favoriter_ids
        FROM
                 articles                 AS a
            JOIN users_author_of_articles AS author ON a.id = author.article_id
            JOIN users                    AS users  ON author.user_id = users.id
            JOIN users_favorite_articles  AS fav    ON a.id = fav.article_id
            JOIN articles_tags            AS a_tags ON a.id = a_tags.article_id
            JOIN tags                     AS tags   ON a_tags.tag_id = tags.id
        GROUP BY
            a.id
    "#).sql());
*/
