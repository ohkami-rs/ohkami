use ohkami::serde::Serialize;
use super::{User, Profile, Article, Comment, Tag};


#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct UserResponse {
    pub user: User,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ProfileResponse {
    pub profile: Profile,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleArticleResponse {
    pub article: Article,
}
#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleArticlesResponse {
    pub articles: Vec<Article>,
    #[serde(rename = "articlesCount")]
    pub articles_count: usize,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ListOfTagsResponse<'t> {
    pub tags: Vec<Tag<'t>>
}
