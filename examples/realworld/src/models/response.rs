use ohkami::typed::Payload;
use ohkami::builtin::payload::JSON;
use super::{User, Profile, Article, Comment, Tag};


#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct UserResponse {
    pub user: User,
}

#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ProfileResponse {
    pub profile: Profile,
}

#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleArticleResponse {
    pub article: Article,
}
#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleArticlesResponse {
    pub articles: Vec<Article>,
    #[serde(rename = "articlesCount")]
    pub articles_count: usize,
}

#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}

#[Payload(JSON/S)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ListOfTagsResponse<'t> {
    pub tags: Vec<Tag<'t>>
}
