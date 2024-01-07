use ohkami::{Ohkami, Route, utils::JSON, http::Status};
use ohkami::utils::{Payload, Query};
use serde::Deserialize;
use crate::fangs::Auth;
use crate::models::{
    Tag,
    Article, SingleArticleResponse, MultipleArticlesResponse,
    Comment, MultipleCommentsResponse, SingleCommentResponse
};


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
struct ArticlesQuery<'q> {
    tag:       Option<&'q str>,
    author:    Option<&'q str>,
    favorited: Option<&'q str>,
    limit:     usize,
    offset:    usize,
}
impl<'q> Default for ArticlesQuery<'q> {
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

async fn list(query: ArticlesQuery<'_>) -> JSON<MultipleArticlesResponse> {
    todo!()
}

async fn feed(query: ArticlesQuery<'_>) -> JSON<MultipleArticlesResponse> {
    todo!()
}

async fn get(slug: &str) -> JSON<SingleArticleResponse> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct CreateArticleRequest<'req> {
    title:         &'req str,
    descipription: &'req str,
    body:          &'req str,
    #[serde(rename = "tagList")]
    tag_list:      Option<Vec<Tag<'req>>>,
}

async fn create(body: CreateArticleRequest<'_>) -> JSON<SingleArticleResponse> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct UpdateArticleRequest<'req> {
    title:       Option<&'req str>,
    description: Option<&'req str>,
    body:        Option<&'req str>,
}

async fn update(slug: &str, body: UpdateArticleRequest<'_>) -> JSON<SingleArticleResponse> {
    todo!()
}

async fn delete(slug: &str) -> Status {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct AddCommentRequest<'req> {
    body: &'req str,
}

async fn add_comment(slug: &str, body: AddCommentRequest<'_>) -> JSON<SingleCommentResponse> {
    todo!()
}

async fn get_comments(slug: &str) -> JSON<MultipleCommentsResponse> {
    todo!()
}

async fn delete_comment((slug, id): (&str, usize)) -> Status {
    todo!()
}

async fn favorite(slug: &str) -> JSON<SingleArticleResponse> {
    todo!()
}

async fn unfavorite(slug: &str) -> JSON<SingleArticleResponse> {
    todo!()
}
