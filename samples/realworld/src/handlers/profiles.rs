use std::borrow::Cow;

use ohkami::prelude::*;
use sqlx::PgPool;
use crate::config::JWTPayload;
use crate::{fangs::Auth, errors::RealWorldError};
use crate::models::response::ProfileResponse;
use crate::db::UserEntity;


pub fn profiles_ohkami() -> Ohkami {
    Ohkami::new((
        Auth::required(),
        "/:username"
            .GET(get_profile),
        "/:username/follow"
            .POST(follow)
            .DELETE(unfollow),
    ))
}

async fn get_profile(username: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>
) -> Result<JSON<ProfileResponse>, RealWorldError> {
    let the_user = UserEntity::get_by_name(username, pool).await?;

    let following_the_user = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users_follow_users AS ufu
            WHERE
                ufu.follower_id = $1 AND
                ufu.followee_id = $2
        )
    "#, auth.user_id, the_user.id)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();

    Ok(JSON(
        the_user.into_profile_response_with(following_the_user)
    ))
}

async fn follow(username: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<ProfileResponse>, RealWorldError> {
    let by_existing_user = sqlx::query!(r#"
        SELECT EXISTS (
            SELECT id
            FROM users AS u
            WHERE
                u.id = $1
        )
    "#, auth.user_id)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?
        .exists.unwrap();
    if !by_existing_user {
        return Err(RealWorldError::Unauthorized(Cow::Owned(format!(
            "User of id '{}' doesn't exist",
            auth.user_id
        ))))
    }

    let followee = UserEntity::get_by_name(username, pool).await?;

    sqlx::query!(r#"
        INSERT INTO
        users_follow_users (followee_id, follower_id)
        VALUES             ($1,          $2)
    "#, followee.id, auth.user_id)
        .execute(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(JSON(
        followee.into_profile_response_with(true)
    ))
}

async fn unfollow(username: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<ProfileResponse>, RealWorldError> {
    let followee = UserEntity::get_by_name(username, pool).await?;

    let deletion_count = sqlx::query!(r#"
        DELETE FROM users_follow_users AS ufu
        WHERE
            ufu.followee_id = $1 AND
            ufu.follower_id = $2
    "#, followee.id, auth.user_id)
        .execute(pool).await
        .map_err(RealWorldError::DB)?
        .rows_affected();
    if deletion_count != 1 {
        tracing::error!("\
            Found {deletion_count} deletion of following \
            {} by {}
        ", followee.id, auth.user_id);
        return Err(RealWorldError::FoundUnexpectedly(Cow::Borrowed(
            "Found more than one following"
        )))
    }

    Ok(JSON(followee.into_profile_response_with(false)))
}
