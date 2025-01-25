use ohkami::prelude::*;
use ohkami::typed::status::{Created, NoContent};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use crate::{config::JWTPayload, errors::RealWorldError};
use crate::fangs::Auth;
use crate::db::{article_id_by_slug, UserAndFollowings, ArticleEntity, CommentEntity};
use crate::models::{
    Article, Profile, Comment,
    request::{ListArticlesQuery, FeedArticleQuery},
    request::{CreateArticleRequest, CreateArticleRequestArticle, UpdateArticleRequest, AddCommentRequest},
    response::{SingleArticleResponse, MultipleArticlesResponse, SingleCommentResponse, MultipleCommentsResponse},
};

pub fn articles_ohkami() -> Ohkami {
    Ohkami::new((
        "/"
            .GET((Auth::optional(), list_articles))
            .POST((Auth::required(), create_article)),
        "/feed"
            .GET((Auth::required(), feed_articles)),
        "/:slug"
            .GET(get_article_by_slug)
            .PUT((Auth::required(), update_article))
            .DELETE((Auth::required(), delete_article)),
        "/:slug/comments"
            .POST((Auth::required(), add_article_comment))
            .GET((Auth::optional(), get_article_comments)),
        "/:slug/comments/:id"
            .DELETE((Auth::required(), delete_article_comment_by_id)),
        "/:slug/favorite"
            .POST((Auth::required(), favorite_article))
            .DELETE((Auth::required(), unfavorite_article))
    ))
}

async fn list_articles(
    Query(q): Query<ListArticlesQuery<'_>>,
    Context(auth): Context<'_, Option<JWTPayload>>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<MultipleArticlesResponse>, RealWorldError> {
    let user_and_followings = match auth {
        None => UserAndFollowings::None,
        Some(JWTPayload { user_id, .. }) => UserAndFollowings::from_user_id(*user_id, pool).await?,
    };

    let articles_data = {
        let mut query = sqlx::QueryBuilder::new(ArticleEntity::base_query());

        let mut once_having = false;
        if let Some(tag) = q.tag {
            query.push(if once_having {" AND "} else {" HAVING "});
            query
                .push_bind(tag)
                .push(" = ANY(ARRAY_AGG(tags.name))");
            once_having = true;
        }
        if let Some(author) = q.author {
            query.push(if once_having {" AND "} else {" HAVING "});
            query
                .push("author.name = ")
                .push_bind(author);
            once_having = true;
        }
        if let Some(favoriter) = q.favorited {
            let favoriter_id = sqlx::query!(r#"
                SELECT id FROM users WHERE name = $1
            "#, favoriter)
                .fetch_one(pool).await
                .map_err(RealWorldError::DB)?
                .id;

            query.push(if once_having {" AND "} else {" HAVING "});
            query
                .push(favoriter_id)
                .push_bind(" = ANY(ARRAY_AGG(fav.user_id))");
        }

        query
            .push(" ORDER BY a.created_at")
            .push(" OFFSET ").push_bind(q.offset())
            .push(" LIMIT ").push_bind(q.limit());

        query.build_query_as::<'_, ArticleEntity>()
            .fetch_all(pool).await
            .map_err(RealWorldError::DB)?
    };

    let articles = articles_data.into_iter()
        .map(|a| a.into_article_with(&user_and_followings))
        .collect::<Vec<_>>();

    Ok(JSON(MultipleArticlesResponse {
        articles_count: articles.len(),
        articles,
    }))
}

async fn feed_articles(
    Query(q): Query<FeedArticleQuery>,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<MultipleArticlesResponse>, RealWorldError> {
    let uf = UserAndFollowings::from_user_id(auth.user_id, pool).await?;

    let articles = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING author.id IN ").push_bind(uf.followings())
        .push(" ORDER BY a.created_at")
        .push(" OFFSET ").push_bind(q.offset())
        .push(" LIMIT ").push_bind(q.limit())
        .build_query_as::<'_, ArticleEntity>()
        .fetch_all(pool).await
        .map_err(RealWorldError::DB)?.into_iter()
        .map(|a| a.into_article_with(&uf)).collect::<Vec<_>>();

    Ok(JSON(MultipleArticlesResponse {
        articles_count: articles.len(),
        articles
    }))
}

async fn get_article_by_slug(slug: &str,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<SingleArticleResponse>, RealWorldError> {
    let article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.slug = ").push_bind(slug)
        .build_query_as::<'_, ArticleEntity>()
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?
        .into_article_with(&UserAndFollowings::None);

    Ok(JSON(SingleArticleResponse {
        article,
    }))
}

async fn create_article(
    JSON(req): JSON<CreateArticleRequest<'_>>,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<Created<JSON<SingleArticleResponse>>, RealWorldError> {
    let author_id = auth.user_id;
    let slug = req.slug();
    let CreateArticleRequest {
        article: CreateArticleRequestArticle {
            title, description, body, tag_list
        }
    } = req;

    let mut tx = pool.begin().await.map_err(RealWorldError::DB)?;

    let created = match sqlx::query!(r#"
        INSERT INTO
            articles (author_id, slug, title, description, body)
            VALUES   ($1,        $2,   $3,    $4,          $5  )
        RETURNING id, created_at, updated_at
    "#, author_id, slug, title, description, body)
        .fetch_one(&mut *tx).await
    {
        Ok(ok) => ok,
        Err(e) => {
            tx.rollback().await.map_err(RealWorldError::DB)?;
            return Err(RealWorldError::DB(e));
        },
    };

    if let Some(tags) = &tag_list {
        /*
            No problem in most cases because, basically,
            `tags` contains at most 4 or 5 items
        */

        let mut tag_ids = Vec::with_capacity(tags.len());
        for tag in tags {
            tag_ids.push(match sqlx::query_scalar!("SELECT id FROM tags WHERE name = $1", &**tag)
                .fetch_optional(pool).await
                .map_err(RealWorldError::DB)?
            {
                Some(existing_id) => existing_id,
                None => sqlx::query_scalar!("INSERT INTO tags (name) VALUES ($1) RETURNING id", &**tag)
                    .fetch_one(pool).await
                    .map_err(RealWorldError::DB)?
            })
        }

        if let Err(e) = sqlx::query!(r#"
            INSERT INTO
                articles_have_tags (tag_id,            article_id       )
                SELECT              UNNEST($1::int[]), UNNEST($2::uuid[])
        "#, &tag_ids, &vec![created.id; tag_ids.len()])
            .execute(&mut *tx).await
        {
            tx.rollback().await.map_err(RealWorldError::DB)?;
            return Err(RealWorldError::DB(e));
        }
    }

    tx.commit().await.map_err(RealWorldError::DB)?;

    let author = sqlx::query!(r#"
        SELECT name, bio, image_url
        FROM users
        WHERE id = $1
    "#, author_id)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(Created(JSON(
        SingleArticleResponse {
            article: Article {
                slug,
                title:           title.into(),
                description:     description.into(),
                body:            body.into(),
                tag_list:        tag_list.unwrap_or_else(Vec::new).into_iter().map(|t| t.to_string()).collect(),
                created_at:      created.created_at,
                updated_at:      created.updated_at,
                favorited:       false,
                favorites_count: 0,
                author: Profile {
                    username:  author.name,
                    bio:       author.bio,
                    image:     author.image_url,
                    following: false  // They doesn't follow themself
                },
            }
        }
    )))
}

async fn update_article(slug: &str,
    JSON(body): JSON<UpdateArticleRequest<'_>>,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<SingleArticleResponse>, RealWorldError> {
    let mut article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.slug = ").push_bind(slug)
        .build_query_as::<ArticleEntity>()
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    if article.author_id != auth.user_id {
        return Err(RealWorldError::Unauthorized(
            std::borrow::Cow::Borrowed("This is not your article")
        ))
    }

    let mut updater = sqlx::QueryBuilder::new("UPDATE articles SET updated_at = now()");
    let mut once_set = false; {
        let set = body.article;

        if let Some(title) = set.title {
            updater
                .push(if once_set {","} else {" SET "})
                .push("title = ").push_bind(title);
            article.title = title.into();
            once_set = true;
        }
        if let Some(description) = set.description {
            updater
                .push(if once_set {","} else {" SET "})
                .push("description = ").push_bind(description);
            article.description = description.into();
            once_set = true;
        }
        if let Some(body) = set.body {
            updater
                .push(if once_set {","} else {" SET "})
                .push("body = ").push_bind(body);
            article.body = body.into();
        }
    }

    article.updated_at = updater
        .push(" WHERE slug = ").push_bind(slug)
        .push(" RETURNING updated_at ")
        .build_query_scalar::<DateTime<Utc>>()
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(JSON(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id, pool).await?
        )
    }))
}

async fn delete_article(slug: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<NoContent, RealWorldError> {
    let n = sqlx::query!("DELETE FROM articles WHERE author_id = $1 AND slug = $2", auth.user_id, slug)
        .execute(pool).await
        .map_err(RealWorldError::DB)?
        .rows_affected();

    match n {
        1 => Ok(NoContent),
        0 => Err(RealWorldError::NotFound(std::borrow::Cow::Borrowed("Article not found"))),
        _ => Err(RealWorldError::FoundUnexpectedly(std::borrow::Cow::Owned(format!("{n} articles deleted"))))
    }
}

async fn add_article_comment(slug: &str,
    JSON(body): JSON<AddCommentRequest<'_>>,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<Created<JSON<SingleCommentResponse>>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug, pool).await?;
    let content = body.comment.content;

    let new_comment_id = sqlx::query_scalar!(r#"
        SELECT id FROM comments
        WHERE article_id = $1
        ORDER BY created_at DESC
        LIMIT 1
    "#, ariticle_id)
        .fetch_optional(pool).await
        .map_err(RealWorldError::DB)?
        .unwrap_or(0) + 1;

    let created_at = sqlx::query_scalar!(r#"
        INSERT INTO
            comments (id, author_id, article_id, content)
            VALUES   ($1, $2,        $3,         $4     )
        RETURNING created_at
    "#, new_comment_id, auth.user_id, ariticle_id, content)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    let comment_author = sqlx::query!(r#"
        SELECT name, bio, image_url
        FROM users
        WHERE id = $1
    "#, auth.user_id)
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(Created(JSON(
        SingleCommentResponse {
            comment: Comment {
                id:         new_comment_id as _,
                created_at: created_at,
                updated_at: created_at,
                body:       content.into(),
                author:     Profile {
                    username:  comment_author.name,
                    bio:       comment_author.bio,
                    image:     comment_author.image_url,
                    following: false,  // They doesn't follow themself
                },
            },
        }
    )))
}

async fn get_article_comments(slug: &str,
    Context(auth): Context<'_, Option<JWTPayload>>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<MultipleCommentsResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug, pool).await?;

    let uf = match auth {
        None => UserAndFollowings::None,
        Some(JWTPayload { user_id, .. }) => UserAndFollowings::from_user_id(*user_id, pool).await?,
    };

    let comments = sqlx::query_as!(CommentEntity, r#"
        SELECT
            c.id,
            c.content,
            c.created_at,
            c.updated_at,
            u.id        AS author_id,
            u.name      AS author_name,
            u.bio       AS author_bio,
            u.image_url AS author_image
        FROM
            comments AS c JOIN
            users    AS u ON c.author_id = u.id
        WHERE
            c.article_id = $1
    "#, ariticle_id)
        .fetch_all(pool).await
        .map_err(RealWorldError::DB)?.into_iter()
        .map(|c| c.into_comment_with(&uf)).collect();

    Ok(JSON(
        MultipleCommentsResponse { comments }
    ))
}

async fn delete_article_comment_by_id((slug, id): (&str, usize),
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<NoContent, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug, pool).await?;

    let n = sqlx::query!(r#"
        DELETE FROM comments
        WHERE
            author_id = $1  AND
            article_id = $2 AND
            id = $3
    "#, auth.user_id, ariticle_id, id as i64)
        .execute(pool).await
        .map_err(RealWorldError::DB)?
        .rows_affected();

    match n {
        1 => Ok(NoContent),
        0 => Err(RealWorldError::NotFound(std::borrow::Cow::Borrowed("Comment not found"))),
        _ => Err(RealWorldError::FoundUnexpectedly(std::borrow::Cow::Owned(format!("{n} comments deleted")))),
    }
}

async fn favorite_article(slug: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<SingleArticleResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug, pool).await?;

    sqlx::query!(r#"
        INSERT INTO
            users_favorite_articles (user_id, article_id)
            VALUES                  ($1,      $2        )
    "#, auth.user_id, ariticle_id)
        .execute(pool).await
        .map_err(RealWorldError::DB)?;

    let article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.id = ").push_bind(ariticle_id)
        .build_query_as::<ArticleEntity>()
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(JSON(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id, pool).await?
        ),
    }))
}

async fn unfavorite_article(slug: &str,
    Context(auth): Context<'_, JWTPayload>,
    Context(pool): Context<'_, PgPool>,
) -> Result<JSON<SingleArticleResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug, pool).await?;

    let n = sqlx::query!(r#"
        DELETE FROM users_favorite_articles
        WHERE
            user_id = $1 AND
            article_id = $2
    "#, auth.user_id, ariticle_id)
        .execute(pool).await
        .map_err(RealWorldError::DB)?
        .rows_affected();

    match n {
        1 => (),
        0 => return Err(RealWorldError::NotFound(std::borrow::Cow::Borrowed("Article not found"))),
        _ => return Err(RealWorldError::FoundUnexpectedly(std::borrow::Cow::Owned(format!("{n} articles found"))))
    }

    let article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.id = ").push_bind(ariticle_id)
        .build_query_as::<ArticleEntity>()
        .fetch_one(pool).await
        .map_err(RealWorldError::DB)?;

    Ok(JSON(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id, pool).await?
        ),
    }))
}
