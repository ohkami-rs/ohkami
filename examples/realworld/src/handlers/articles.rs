use ohkami::{Ohkami, Route, http::Status, typed::{OK, Created}, Memory};
use ohkami::utils::{Payload, Query};
use sqlx::Execute;
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
    let user_id = auth.as_ref().map(|jwt| jwt.user_id);

    let mut query = sqlx::QueryBuilder::new(sqlx::query_as!(ArticleEntity, r#"
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
            JSON_AGG(users)        AS authors
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
    query
        .push(" ORDER BY a.created_at")
        .push(" OFFSET ").push_bind(q.offset())
        .push(" LIMIT ").push_bind(q.limit());
    query.build().execute(pool()).await.map_err(RealWorldError::DB)?;


    /* `author.following` は上記とは別のクエリで取得する */
    /*
        - user_id が None のとき (つまりログインしていないとき) は常に author.following = false,
        - user_id が Some のときは authoer.following = {
            SELECT EXISTS users_follow_users AS ufu
            WHERE ufu.follower_id = user_id AND ufu.followee_id = author.id
        }

        (実際には IN array を使い、レスポンスに含める全ての author についての following
        を一度のクエリで取得する)
    */

    todo!()
}

#[Query]
struct FeedArticleQuery {
    limit:  Option<usize>,
    offset: Option<usize>,
}

async fn feed(query: FeedArticleQuery) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    unimplemented!()
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
