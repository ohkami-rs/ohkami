use chrono::{DateTime, Utc};
use ohkami::{Ohkami, Route, typed::{OK, Created, NoContent}, Memory};
use crate::{config::{JWTPayload, pool}, errors::RealWorldError};
use crate::fangs::{Auth, OptionalAuth};
use crate::db::{article_id_by_slug, UserAndFollowings, ArticleEntity, CommentEntity};
use crate::models::request::{
    ListArticlesQuery,
    FeedArticleQuery,
};
use crate::models::{
    Article, Profile, Comment,
    request::{CreateArticleRequest, UpdateArticleRequest, AddCommentRequest},
    response::{SingleArticleResponse, MultipleArticlesResponse, SingleCommentResponse, MultipleCommentsResponse},
};


pub fn articles_ohkami() -> Ohkami {
    fn auth_required(req: &ohkami::Request) -> bool {
        (!req.method.isGET()) || req.path().ends_with("/feed")
    }

    Ohkami::with((
        Auth        ::with_condition(|req| auth_required(req)),
        OptionalAuth::with_condition(|req| ! auth_required(req)),
    ), (//auth:
        "/"
            .GET(list)//optional
            .POST(create),//required
        "/feed"
            .GET(feed),//required
        "/:slug".By(Ohkami::new((
            "/"
                .GET(get)//no
                .PUT(update)//required
                .DELETE(delete),//required
            "/comments"
                .POST(add_comment)//required
                .GET(get_comments),//optional
            "/comments/:id"
                .DELETE(delete_comment),//required
            "/favorite"
                .POST(favorite)//required
                .DELETE(unfavorite)//required
        )))
    ))
}


async fn list(
    q:    ListArticlesQuery<'_>,
    auth: Memory<'_, Option<JWTPayload>>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    let user_and_followings = match *auth {
        None => UserAndFollowings::None,
        Some(JWTPayload { user_id, .. }) => UserAndFollowings::from_user_id(*user_id).await?,
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
                .fetch_one(pool()).await
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
            .fetch_all(pool()).await
            .map_err(RealWorldError::DB)?
    };

    let articles = articles_data.into_iter()
        .map(|a| a.into_article_with(&user_and_followings))
        .collect::<Vec<_>>();

    Ok(OK(MultipleArticlesResponse {
        articles_count: articles.len(),
        articles,
    }))
}

async fn feed(
    q:    FeedArticleQuery,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    let uf = UserAndFollowings::from_user_id(auth.user_id).await?;

    let articles = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING author.id IN ").push_bind(uf.followings())
        .push(" ORDER BY a.created_at")
        .push(" OFFSET ").push_bind(q.offset())
        .push(" LIMIT ").push_bind(q.limit())
        .build_query_as::<'_, ArticleEntity>()
        .fetch_all(pool()).await
        .map_err(RealWorldError::DB)?.into_iter()
        .map(|a| a.into_article_with(&uf)).collect::<Vec<_>>();

    Ok(OK(MultipleArticlesResponse {
        articles_count: articles.len(),
        articles
    }))
}

async fn get(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    let article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.slug = ").push_bind(slug)
        .build_query_as::<'_, ArticleEntity>()
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?
        .into_article_with(&UserAndFollowings::None);

    Ok(OK(SingleArticleResponse {
        article,
    }))
}

async fn create(
    auth: Memory<'_, JWTPayload>,
    req:  CreateArticleRequest<'_>,
) -> Result<Created<SingleArticleResponse>, RealWorldError> {
    let author_id = auth.user_id;
    let slug = req.slug();
    let CreateArticleRequest { title, description, body, tag_list } = req;

    let created = sqlx::query!(r#"
        INSERT INTO
            articles (author_id, slug, title, description, body)
            VALUES   ($1,        $2,   $3,    $4,          $5  )
        RETURNING id, created_at, updated_at
    "#, author_id, slug, title, description, body)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    if let Some(tags) = &tag_list {
        /*
            No problem in most cases because, basically,
            `tags` contains at most 4 or 5 items
        */

        let mut tag_ids = Vec::with_capacity(tags.len());
        for tag in tags {
            tag_ids.push(match sqlx::query_scalar!("SELECT id FROM tags WHERE name = $1", &**tag)
                .fetch_optional(pool()).await
                .map_err(RealWorldError::DB)?
            {
                Some(existing_id) => existing_id,
                None => sqlx::query_scalar!("INSERT INTO tags (name) VALUES ($1) RETURNING id", &**tag)
                    .fetch_one(pool()).await
                    .map_err(RealWorldError::DB)?
            })
        }

        sqlx::query!(r#"
            INSERT INTO
                articles_tags (tag_id,            article_id       )
                SELECT        UNNEST($1::uuid[]), UNNEST($2::uuid[])
        "#, &tag_ids, &vec![created.id; tag_ids.len()])
            .execute(pool()).await
            .map_err(RealWorldError::DB)?;
    }

    let author = sqlx::query!(r#"
        SELECT name, bio, image_url
        FROM users
        WHERE id = $1
    "#, author_id)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(Created(SingleArticleResponse {
        article: Article {
            title:           title.into(),
            slug:            Some(slug),
            description:     Some(description.into()),
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
    }))
}

async fn update(
    slug: &str,
    body: UpdateArticleRequest<'_>,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    let mut article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.slug = ").push_bind(slug)
        .build_query_as::<ArticleEntity>()
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    if article.author_id != auth.user_id {
        return Err(RealWorldError::Unauthorized(
            std::borrow::Cow::Borrowed("This is not your article")
        ))
    }

    let mut updater = sqlx::QueryBuilder::new("UPDATE articles SET updated_at = now()");
    let mut once_set = false; {
        if let Some(title) = body.title {
            updater
                .push(if once_set {","} else {" SET "})
                .push("title = ").push_bind(title);
            article.title = title.into();
            once_set = true;
        }
        if let Some(description) = body.description {
            updater
                .push(if once_set {","} else {" SET "})
                .push("description = ").push_bind(description);
            article.description = Some(description.into());
            once_set = true;
        }
        if let Some(body) = body.body {
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
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(OK(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id).await?
        )}))
}

async fn delete(
    slug: &str,
    auth: Memory<'_, JWTPayload>,
) -> Result<NoContent, RealWorldError> {
    let n = sqlx::query!("DELETE FROM articles WHERE author_id = $1 AND slug = $2", auth.user_id, slug)
        .execute(pool()).await
        .map_err(RealWorldError::DB)?
        .rows_affected();

    match n {
        1 => Ok(NoContent),
        0 => Err(RealWorldError::NotFound(std::borrow::Cow::Borrowed("Article not found"))),
        _ => Err(RealWorldError::FoundUnexpectedly(std::borrow::Cow::Owned(format!("{n} articles deleted"))))
    }
}

async fn add_comment(
    slug: &str,
    body: AddCommentRequest<'_>,
    auth: Memory<'_, JWTPayload>,
) -> Result<Created<SingleCommentResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug).await?;

    let created = sqlx::query!(r#"
        INSERT INTO
            comments (author_id, article_id, content)
            VALUES   ($1,        $2,         $3     )
        RETURNING id, created_at
    "#, auth.user_id, ariticle_id, body.content)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    let comment_author = sqlx::query!(r#"
        SELECT name, bio, image_url
        FROM users
        WHERE id = $1
    "#, auth.user_id)
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(Created(SingleCommentResponse {
        comment: Comment {
            id:         created.id as _,
            created_at: created.created_at,
            updated_at: created.created_at,
            body:       body.content.into(),
            author:     Profile {
                username:  comment_author.name,
                bio:       comment_author.bio,
                image:     comment_author.image_url,
                following: false,  // They doesn't follow themself
            },
        },
    }))
}

async fn get_comments(
    slug: &str,
    auth: Memory<'_, Option<JWTPayload>>,
) -> Result<OK<MultipleCommentsResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug).await?;

    let uf = match *auth {
        None => UserAndFollowings::None,
        Some(JWTPayload { user_id, .. }) => UserAndFollowings::from_user_id(*user_id).await?,
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
        .fetch_all(pool()).await
        .map_err(RealWorldError::DB)?.into_iter()
        .map(|c| c.into_comment_with(&uf)).collect();

    Ok(OK(MultipleCommentsResponse { comments }))
}

async fn delete_comment(
    (slug, id): (&str, usize),
    auth: Memory<'_, JWTPayload>,
) -> Result<NoContent, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug).await?;

    let n = sqlx::query!(r#"
        DELETE FROM comments
        WHERE
            author_id = $1  AND
            article_id = $2 AND
            id = $3
    "#, auth.user_id, ariticle_id, id as i64)
        .execute(pool()).await
        .map_err(RealWorldError::DB)?
        .rows_affected();

    match n {
        1 => Ok(NoContent),
        0 => Err(RealWorldError::NotFound(std::borrow::Cow::Borrowed("Comment not found"))),
        _ => Err(RealWorldError::FoundUnexpectedly(std::borrow::Cow::Owned(format!("{n} comments deleted")))),
    }
}

async fn favorite(
    slug: &str,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug).await?;

    sqlx::query!(r#"
        INSERT INTO
            users_favorite_articles (user_id, article_id)
            VALUES                  ($1,      $2        )
    "#, auth.user_id, ariticle_id)
        .execute(pool()).await
        .map_err(RealWorldError::DB)?;

    let article = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING a.id = ").push_bind(ariticle_id)
        .build_query_as::<ArticleEntity>()
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(OK(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id).await?
        ),
    }))
}

async fn unfavorite(
    slug: &str,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    let ariticle_id = article_id_by_slug(slug).await?;

    let n = sqlx::query!(r#"
        DELETE FROM users_favorite_articles
        WHERE
            user_id = $1 AND
            article_id = $2
    "#, auth.user_id, ariticle_id)
        .execute(pool()).await
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
        .fetch_one(pool()).await
        .map_err(RealWorldError::DB)?;

    Ok(OK(SingleArticleResponse {
        article: article.into_article_with(
            &UserAndFollowings::from_user_id(auth.user_id).await?
        ),
    }))
}
