use ohkami::typed::ResponseBody;
use super::{User, Profile, Article, Comment, Tag};


#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct UserResponse {
    pub user: User,
}

#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ProfileResponse {
    pub profile: Profile,
}

#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleArticleResponse {
    pub article: Article,
}
#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleArticlesResponse {
    pub articles: Vec<Article>,
    #[serde(rename = "articlesCount")]
    pub articles_count: usize,
}

#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}

#[ResponseBody(JSONS)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct ListOfTagsResponse<'t> {
    pub tags: Vec<Tag<'t>>
}
