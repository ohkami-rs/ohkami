use ohkami::{Ohkami, Route, http::Status, typed::{OK, Created}, Memory};
use ohkami::utils::{Payload, Query};
use sqlx::Execute;
use crate::{errors::RealWorldError, config::{JWTPayload, pool}};
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
struct ArticlesQuery<'q> {
    tag:       Option<&'q str>,
    author:    Option<&'q str>,
    favorited: Option<&'q str>,
    limit:     Option<usize>,
    offset:    Option<usize>,
} impl<'q> ArticlesQuery<'q> {
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as _
    }
    fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as _
    }
}

async fn list(
    q:    ArticlesQuery<'_>,
    auth: Memory<'_, Option<JWTPayload>>,
) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    let user_id = auth.as_ref().map(|jwt| jwt.user_id);

    let mut query = sqlx::QueryBuilder::new(sqlx::query!(r#"
        SELECT
            a.id          AS id,
            a.slug        AS article_slug,
            a.title       AS article_title,
            a.description AS article_description,
            a.body        AS article_body,
            a.created_at  AS article_created_at,
            a.updated_at  AS article_updated_at,
            u.id          AS user_id,
            u.email       AS user_email,
            u.name        AS user_name,
            u.bio         AS user_bio,
            u.image_url   AS user_image
        FROM
                 articles                AS a
            JOIN users                   AS u    ON a.author_id = u.id
            JOIN users_favorite_articles AS fav  ON a.id = fav.article_id
            JOIN articles_tags           AS tags ON a.id = tags.article_id
        GROUP BY
            a.id, u.id
    "#).sql());



    query
        .push(" ORDER BY a.created_at")
        .push(" OFFSET ").push_bind(q.offset())
        .push(" LIMIT ").push_bind(q.limit());



    query.build().execute(pool()).await.map_err(RealWorldError::DB)?;

    todo!()
}

async fn feed(query: ArticlesQuery<'_>) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    todo!()
}

async fn get(slug: &str) -> Result<OK<SingleArticleResponse>, RealWorldError> {
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
