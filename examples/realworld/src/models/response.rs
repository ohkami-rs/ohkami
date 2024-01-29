use ohkami::utils::ResponseBody;
use super::{User, Profile, Article, Comment, Tag};


#[ResponseBody(JSONS)]
pub struct UserResponse {
    pub user: User,
}

#[ResponseBody(JSONS)]
pub struct ProfileResponse {
    pub profile: Profile,
}

#[ResponseBody(JSONS)]
pub struct SingleArticleResponse {
    pub article: Article,
}
#[ResponseBody(JSONS)]
pub struct MultipleArticlesResponse {
    pub articles: Vec<Article>,
    #[serde(rename = "articlesCount")]
    pub articles_count: usize,
}

#[ResponseBody(JSONS)]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[ResponseBody(JSONS)]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}

#[ResponseBody(JSONS)]
pub struct ListOfTagsResponse<'t> {
    pub tags: Vec<Tag<'t>>
}
