use std::borrow::{BorrowMut, Cow};
use ohkami::{Ohkami, Route, http::Status, typed::{OK, Created, NoContent}, Memory};
use ohkami::utils::{Payload, Query};
use sqlx::Execute;
use uuid::Uuid;
use crate::{config::{JWTPayload, pool}, db::ArticleEntity, errors::RealWorldError, models::Profile};
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
            let followings = sqlx::query_scalar!(r#"
                SELECT followee_id
                FROM users_follow_users
                WHERE follower_id = $1
            "#, user_id)
                .fetch_all(pool()).await
                .map_err(RealWorldError::DB)?;

            Some((*user_id, followings))
        }
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
        .map(|a| a.into_article_with(user_and_followings.as_ref()))
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
} impl FeedArticleQuery {
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as _
    }
    fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as _
    }
}

async fn feed(
    q:    FeedArticleQuery,
    auth: Memory<'_, JWTPayload>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    let followings = sqlx::query_scalar!(r#"
        SELECT followee_id
        FROM users_follow_users
        WHERE follower_id = $1
    "#, auth.user_id)
        .fetch_all(pool()).await
        .map_err(RealWorldError::DB)?;

    if followings.is_empty() {
        return Ok(OK(MultipleArticlesResponse {
            articles: Vec::new(),
            articles_count: 0,
        }))
    }

    let articles = sqlx::QueryBuilder::new(ArticleEntity::base_query())
        .push(" HAVING author.id IN ").push_bind(&followings)
        .push(" ORDER BY a.created_at")
        .push(" OFFSET ").push_bind(q.offset())
        .push(" LIMIT ").push_bind(q.limit())
        .build_query_as::<'_, ArticleEntity>()
        .fetch_all(pool()).await
        .map_err(RealWorldError::DB)?.into_iter()
        .map(|a| a.into_article_with(Some(&(auth.user_id, &followings)))).collect::<Vec<_>>();

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
        .into_article();

    Ok(OK(SingleArticleResponse {
        article,
    }))
}

#[Payload(JSOND)]
struct CreateArticleRequest<'req> {
    title:       &'req str,
    description: &'req str,
    body:        &'req str,
    #[serde(rename = "tagList")]
    tag_list:      Option<Vec<Tag<'req>>>,
} impl CreateArticleRequest<'_> {
    fn slug(&self) -> String {
        self.title.chars().filter_map(|ch| match ch {
            '/' | '?' | '=' | '&' | '#'     => None,
            ' ' | '　' | '\r' | '\n' | '\t' => Some('-'),
            _ => Some(ch)
        }).collect()
    }
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
                following: false  // They doesn't follow themselves
            },
        }
    }))
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

async fn delete(slug: &str) -> Result<NoContent, RealWorldError> {


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

async fn delete_comment((slug, id): (&str, usize)) -> Result<NoContent, RealWorldError> {
    todo!()
}

async fn favorite(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    todo!()
}

async fn unfavorite(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
    todo!()
}
