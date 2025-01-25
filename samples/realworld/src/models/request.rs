use ohkami::serde::Deserialize;
use super::Tag;


#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct LoginRequest<'req> {
    #[serde(borrow)]
    pub user: LoginRequestUser<'req>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct LoginRequestUser<'req> {
    pub email:    &'req str,
    pub password: &'req str,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct RegisterRequest<'req> {
    #[serde(borrow)]
    pub user: RegisterRequestUser<'req>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct RegisterRequestUser<'req> {
    pub username: &'req str,
    pub email:    &'req str,
    pub password: &'req str,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct UpdateProfileRequest<'req> {
    #[serde(borrow)]
    pub user: UpdateProfileRequestUser<'req>
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct UpdateProfileRequestUser<'req> {
    pub email:    Option<&'req str>,
    pub username: Option<&'req str>,
    pub password: Option<&'req str>,
    pub image:    Option<&'req str>,
    pub bio:      Option<&'req str>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct ListArticlesQuery<'q> {
    pub tag:       Option<&'q str>,
    pub author:    Option<&'q str>,
    pub favorited: Option<&'q str>,
    pub limit:     Option<usize>,
    pub offset:    Option<usize>,
} impl<'q> ListArticlesQuery<'q> {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as _
    }
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as _
    }
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct FeedArticleQuery {
    limit:  Option<usize>,
    offset: Option<usize>,
} impl FeedArticleQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20) as _
    }
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0) as _
    }
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct CreateArticleRequest<'req> {
    #[serde(borrow)]
    pub article: CreateArticleRequestArticle<'req>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct CreateArticleRequestArticle<'req> {
    pub title:       &'req str,
    pub description: &'req str,
    pub body:        &'req str,
    #[serde(rename = "tagList")]
    pub tag_list:    Option<Vec<Tag<'req>>>,
}
impl CreateArticleRequest<'_> {
    pub fn slug(&self) -> String {
        self.article.title.chars().filter_map(|ch| match ch {
            '/' | '?' | '=' | '&' | '#'     => None,
            ' ' | '　' | '\r' | '\n' | '\t' => Some('-'),
            _ => Some(ch)
        }).collect()
    }
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct UpdateArticleRequest<'req> {
    #[serde(borrow)]
    pub article: UpdateArticleRequestArticle<'req>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct UpdateArticleRequestArticle<'req> {
    pub title:       Option<&'req str>,
    pub description: Option<&'req str>,
    pub body:        Option<&'req str>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct AddCommentRequest<'req> {
    #[serde(borrow)]
    pub comment: AddCommentRequestComment<'req>,
}
#[derive(Deserialize)]
#[cfg_attr(test, derive(ohkami::serde::Serialize))]
pub struct AddCommentRequestComment<'req> {
    #[serde(rename = "body")]
    pub content: &'req str,
}
