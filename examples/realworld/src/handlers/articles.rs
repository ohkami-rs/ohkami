use ohkami::{Ohkami, Route, http::Status, typed::{OK, Created}};
use ohkami::utils::{Payload, Query};
use crate::{fangs::Auth, errors::RealWorldError};
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
    limit:     Option<usize>,
    offset:    Option<usize>,
} impl<'q> ArticlesQuery<'q> {
    fn limit(&self) -> usize {
        self.limit.unwrap_or(20)
    }
    fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }
}

async fn list(query: ArticlesQuery<'_>) -> Result<OK<MultipleArticlesResponse>, RealWorldError> {
    

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
