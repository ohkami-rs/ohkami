use crate::{config, errors::RealWorldError};
use crate::models::{Article, Profile, User, Comment};
use crate::models::response::{UserResponse, ProfileResponse};
use argon2::{Algorithm, Argon2, Params, PasswordHasher,  Version};
use argon2::password_hash::{PasswordHashString, Salt, SaltString};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;


pub fn hash_password(
    raw_password: &str,
) -> Result<(PasswordHashString, SaltString), RealWorldError> {
    let salt = SaltString::generate(::argon2::password_hash::rand_core::OsRng);
    let hash = __hash_password_with(raw_password, &salt)?;

    Ok((hash, salt))
}

pub fn verify_password(
    raw_password:    &str,
    salt:            &str,
    hashed_password: &str,
) -> Result<(), RealWorldError> {
    let correct_hash = __hash_password_with(
        raw_password,
        &SaltString::from_b64(salt).unwrap(),
    )?;

    if correct_hash.as_str() != hashed_password {
        return Err(RealWorldError::Unauthorized(std::borrow::Cow::Borrowed(
            "Wrong email or password"
        )))
    }

    Ok(())
}

fn __hash_password_with(
    raw_password: &str,
    salt: &SaltString,
) -> Result<PasswordHashString, RealWorldError> {
    let pepper = config::PEPPER()?;

    let a = Argon2::new_with_secret(
        pepper.as_bytes(),
        Algorithm::Argon2id,
        Version::V0x13,
        Params::DEFAULT,
    ).map_err(|e| RealWorldError::Config(e.to_string()))?;

    let hash = a.hash_password(
        raw_password.as_bytes(),
        Salt::from(salt),
    ).map_err(|e| RealWorldError::Config(e.to_string()))?;

    Ok(hash.serialize())
}

pub async fn article_id_by_slug(slug: &str, pool: &PgPool) -> Result<Uuid, RealWorldError> {
    sqlx::query_scalar!(r#"
        SELECT id
        FROM articles
        WHERE slug = $1
    "#, slug)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)
}

pub enum UserAndFollowings {
    None,
    Some { user: Uuid, followings: Vec<Uuid> },
} impl UserAndFollowings {
    pub fn user_id(&self) -> Option<&Uuid> {
        match self {
            Self::None => None,
            Self::Some { user, .. } => Some(user),
        }
    }
    pub fn followings(&self) -> &[Uuid] {
        match self {
            Self::None => &[],
            Self::Some { followings, .. } => followings,
        }
    }
} impl UserAndFollowings {
    pub async fn from_user_id(user_id: Uuid, pool: &PgPool) -> Result<Self, RealWorldError> {
        let followings = sqlx::query_scalar!(r#"
            SELECT followee_id FROM users_follow_users WHERE follower_id = $1
        "#, user_id)
            .fetch_all(pool).await
            .map_err(RealWorldError::DB)?;

        Ok(Self::Some {
            user: user_id,
            followings
        })
    }
}

#[derive(sqlx::FromRow)]
pub struct UserEntity {
    pub id:        Uuid,
    pub email:     String,
    pub name:      String,
    pub bio:       Option<String>,
    pub image_url: Option<String>,
} impl UserEntity {
    pub fn into_user(self) -> Result<User, RealWorldError> {
        Ok(User {
            jwt:   config::issue_jwt_for_user_of_id(self.id)?,
            email: self.email,
            name:  self.name,
            bio:   self.bio,
            image: self.image_url,
        })
    }
    pub fn into_user_response(self) -> Result<UserResponse, RealWorldError> {
        Ok(UserResponse {
            user: self.into_user()?
        })
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
    pub async fn get_by_name(name: &str, pool: &PgPool) -> Result<Self, RealWorldError> {
        sqlx::query_as!(UserEntity, r#"
            SELECT id, name, email, bio, image_url
            FROM users AS u
            WHERE
                u.name = $1
        "#, name)
            .fetch_one(pool).await
            .map_err(RealWorldError::DB)
    }
}

#[derive(sqlx::FromRow)]
pub struct ArticleEntity {
    #[allow(unused)]
    pub id:              Uuid,
    pub slug:            String,
    pub title:           String,
    pub description:     String,
    pub body:            String,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    pub favorites_count: Option<i64>,
    pub favoriter_ids:   Option<Vec<Uuid>>,
    pub tags:            Option<Vec<String>>,
    pub author_id:       Uuid,
    pub author_name:     String,
    pub author_bio:      Option<String>,
    pub author_image:    Option<String>,
} impl ArticleEntity {
    #[cfg_attr(not(test), inline(always))]
    pub fn base_query() -> &'static str {
        macro_rules! verified {
            ($query:literal) => {
                {
                    #[cfg(test)] {
                        use sqlx::Execute;
                        sqlx::query_as!(Self, $query).sql();
                    }
                    
                    $query
                }
            };
        }

        verified!(r#"
            SELECT
                a.id                   AS id,
                a.slug                 AS slug,
                a.title                AS title,
                a.description          AS description,
                a.body                 AS body,
                a.created_at           AS created_at,
                a.updated_at           AS updated_at,
                COUNT(fav.id)          AS favorites_count,
                ARRAY_AGG(fav.user_id) AS favoriter_ids,
                ARRAY_AGG(tags.name)   AS tags,
                author.id              AS author_id,
                author.name            AS author_name,
                author.bio             AS author_bio,
                author.image_url       AS author_image
            FROM
                     articles                 AS a
                JOIN users                    AS author ON a.author_id = author.id
                JOIN users_favorite_articles  AS fav    ON a.id = fav.article_id
                JOIN articles_have_tags       AS a_tags ON a.id = a_tags.article_id
                JOIN tags                     AS tags   ON a_tags.tag_id = tags.id
            GROUP BY
                a.id, author.id
        "#)
    }
} impl ArticleEntity {
    pub fn into_article_with(
        self,
        uf: &UserAndFollowings,
    ) -> Article {
        let favorited = uf.user_id()
            .map(|id| self.favoriter_ids.unwrap_or_else(Vec::new).contains(id))
            .unwrap_or(false);

        let author = Profile {
            username:  self.author_name,
            bio:       self.author_bio,
            image:     self.author_image,
            following: uf.followings().contains(&self.author_id),
        };

        Article {
            title:           self.title,
            description:     self.description,
            body:            self.body,
            slug:            self.slug,
            created_at:      self.created_at,
            updated_at:      self.updated_at,
            tag_list:        self.tags.unwrap_or_else(Vec::new),
            favorites_count: self.favorites_count.unwrap_or(0) as _,
            favorited,
            author,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct CommentEntity {
    pub id:           i32,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
    pub content:      String,
    pub author_id:    Uuid,
    pub author_name:  String,
    pub author_bio:   Option<String>,
    pub author_image: Option<String>,
} impl CommentEntity {
    pub fn into_comment_with(self, uf: &UserAndFollowings) -> Comment {
        Comment {
            id:         self.id as _,
            created_at: self.created_at,
            updated_at: self.updated_at,
            body:       self.content,
            author:     Profile {
                username:  self.author_name,
                bio:       self.author_bio,
                image:     self.author_image,
                following: uf.followings().contains(&self.author_id),
            },
        }
    }
}
