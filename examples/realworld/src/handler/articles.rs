use ohkami::{Ohkami, Route, Context, Response};
use ohkami::utils::{Payload, Query};
use serde::Deserialize;
use crate::fangs::Auth;


pub fn articles_ohkami() -> Ohkami {
    Ohkami::with((), (
        "/"
            .GET(list)//optional
            .POST(create),//required
        "/feed"
            .GET(feed),//required
        "/:slug"
            .GET(get)//no
            .PUT(update)//required
            .DELETE(delete),//required
        "/:slug/comments"
            .POST(add_comment)//required
            .GET(get_comments),//optional
        "/:slug/comments/:id"
            .DELETE(delete_comment),//required
        "/:slug/favorite"
            .POST(favorite)//required
            .DELETE(unfavorite)//required
    ))
}

#[Query]
struct ArticlesQuery {
    tag:       Option<String>,
    author:    Option<String>,
    favorited: Option<String>,
    limit:     usize,
    offset:    usize,
}
impl Default for ArticlesQuery {
    fn default() -> Self {
        ArticlesQuery {
            tag:       None,
            author:    None,
            favorited: None,
            limit:     20,
            offset:    0,
        }
    }
}

async fn list(c: Context, query: ArticlesQuery) -> Response {
    todo!()
}

async fn feed(c: Context, query: ArticlesQuery) -> Response {
    todo!()
}

async fn get(c: Context, slug: String) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct CreateArticleRequest {
    title:         String,
    descipription: String,
    body:          String,
    #[serde(rename = "tagList")]
    tag_list:      Option<Vec<String>>,
}

async fn create(c: Context, body: CreateArticleRequest) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct UpdateArticleRequest {
    title:       Option<String>,
    description: Option<String>,
    body:        Option<String>,
}

async fn update(c: Context, slug: String, body: UpdateArticleRequest) -> Response {
    todo!()
}

async fn delete(c: Context, slug: String) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct AddCommentRequest {
    body: String,
}

async fn add_comment(c: Context, slug: String, body: AddCommentRequest) -> Response {
    todo!()
}

async fn get_comments(c: Context, slug: String) -> Response {
    todo!()
}

async fn delete_comment(c: Context, (slug, id): (String, usize)) -> Response {
    todo!()
}

async fn favorite(c: Context, slug: String) -> Response {
    todo!()
}

async fn unfavorite(c: Context, slug: String) -> Response {
    todo!()
}
