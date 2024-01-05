use ohkami::{Ohkami, Route, Response};
use ohkami::utils::{Payload, Query};
use serde::Deserialize;
use crate::fangs::Auth;


pub fn articles_ohkami() -> Ohkami {
    Ohkami::with((
        Auth::with_condition(|req| (!req.method.isGET()) || req.path().ends_with("/feed")),
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

async fn list(query: ArticlesQuery) -> Response {
    todo!()
}

async fn feed(query: ArticlesQuery) -> Response {
    todo!()
}

async fn get(slug: String) -> Response {
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

async fn create(body: CreateArticleRequest) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct UpdateArticleRequest {
    title:       Option<String>,
    description: Option<String>,
    body:        Option<String>,
}

async fn update(slug: String, body: UpdateArticleRequest) -> Response {
    todo!()
}

async fn delete(slug: String) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct AddCommentRequest {
    body: String,
}

async fn add_comment(slug: String, body: AddCommentRequest) -> Response {
    todo!()
}

async fn get_comments(slug: String) -> Response {
    todo!()
}

async fn delete_comment((slug, id): (String, usize)) -> Response {
    todo!()
}

async fn favorite(slug: String) -> Response {
    todo!()
}

async fn unfavorite(slug: String) -> Response {
    todo!()
}
