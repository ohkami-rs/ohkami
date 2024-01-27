use std::borrow::Cow;
use ohkami::{Ohkami, Route, http::Status, typed::{OK, Created}, Memory};
use ohkami::utils::{Payload, Query};
use sqlx::Execute;
use uuid::Uuid;
use crate::{errors::RealWorldError, config::{JWTPayload, pool}, db::ArticleEntity};
use crate::fangs::{Auth, OptionalAuth};
use crate::models::{
    Tag,
    Article, SingleArticleResponse, MultipleArticlesResponse,
    Comment, MultipleCommentsResponse, SingleCommentResponse
};


pub fn articles_ohkami() -> Ohkami {
    fn auth_required(req: &ohkami::Request) -> bool {
        (!req.method.isGET()) || req.path().ends_with("/feed")
    }

    Ohkami::with((
        Auth        ::with_condition(|req| auth_required(req)),
        OptionalAuth::with_condition(|req| ! auth_required(req)),
    ), (
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


#[Query]
struct ListArticlesQuery<'q> {
    tag:       Option<&'q str>,
    author:    Option<&'q str>,
    favorited: Option<&'q str>,
    limit:     Option<usize>,
    offset:    Option<usize>,
} impl<'q> ListArticlesQuery<'q> {
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as _
    }
    fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as _
    }
}

async fn list(
    q:    ListArticlesQuery<'_>,
    auth: Memory<'_, Option<JWTPayload>>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    let user_and_followings: Option<(Uuid, Vec<Uuid>)> = match *auth {
        None => None,
        Some(JWTPayload { user_id, .. }) => {
            let followings = sqlx::query!(r#"
                SELECT followee_id
                FROM users_follow_users
                WHERE follower_id = $1
            "#, user_id)
                .fetch_all(pool()).await
                .map_err(RealWorldError::DB)?.into_iter()
                .map(|r| r.followee_id).collect::<Vec<_>>();

            Some((*user_id, followings))
        }
    };

    let articles_data = {
        let mut query = sqlx::QueryBuilder::<'_, sqlx::Postgres>::new(sqlx::query_as!(ArticleEntity, r#"
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
                JOIN users                    AS author  ON a.author_id = author.id
                JOIN users_favorite_articles  AS fav     ON a.id = fav.article_id
                JOIN articles_tags            AS a_tags  ON a.id = a_tags.article_id
                JOIN tags                     AS tags    ON a_tags.tag_id = tags.id
            GROUP BY
                a.id, author.id
        "#).sql());

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

#[Query]
struct FeedArticleQuery {
    limit:  Option<usize>,
    offset: Option<usize>,
}

async fn feed(
    q:    FeedArticleQuery,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    todo!()
}

async fn get(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    // let;

    todo!()
}

#[Payload(JSOND)]
struct CreateArticleRequest<'req> {
    title:         &'req str,
    descipription: &'req str,
    body:          &'req str,
    #[serde(rename = "tagList")]
    tag_list:      Option<Vec<Tag<'req>>>,
}

async fn create(body: CreateArticleRequest<'_>) -> Result<Created<SingleArticleResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSOND)]
struct UpdateArticleRequest<'req> {
    title:       Option<&'req str>,
    description: Option<&'req str>,
    body:        Option<&'req str>,
}

async fn update(slug: &str, body: UpdateArticleRequest<'_>) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    todo!()
}

async fn delete(slug: &str) -> Status {
    todo!()
}

#[Payload(JSOND)]
struct AddCommentRequest<'req> {
    body: &'req str,
}

async fn add_comment(slug: &str, body: AddCommentRequest<'_>) -> Result<Created<SingleCommentResponse>, RealWorldError> {
    todo!()
}

async fn get_comments(slug: &str) -> Result<OK<MultipleCommentsResponse>, RealWorldError> {
    todo!()
}

async fn delete_comment((slug, id): (&str, usize)) -> Status {
    todo!()
}

async fn favorite(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    todo!()
}

async fn unfavorite(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    todo!()
}
