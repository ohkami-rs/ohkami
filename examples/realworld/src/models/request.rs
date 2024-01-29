use ohkami::utils::{Deserialize, Deserializer, Payload, Query};
use super::Tag;


#[Payload(JSON)]
pub struct LoginRequest<'req> {
    pub user: LoginRequestUser<'req>,
} const _: () = {
    impl<'req> Deserialize<'req> for LoginRequest<'req> {
        fn deserialize<D: Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Self {
                user: LoginRequestUser::deserialize(deserializer)?,
            })
        }
    }
};
#[derive(Deserialize)]
pub struct LoginRequestUser<'req> {
    pub email:    &'req str,
    pub password: &'req str,
}

#[Payload(JSOND)]
pub struct RegisterRequest<'req> {
    pub username: &'req str,
    pub email:    &'req str,
    pub password: &'req str,
}

#[Payload(JSOND)]
pub struct UpdateProfileRequest {
    pub email:    Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub image:    Option<String>,
    pub bio:      Option<String>,
}

#[Query]
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

#[Query]
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

#[Payload(JSOND)]
pub struct CreateArticleRequest<'req> {
    pub title:       &'req str,
    pub description: &'req str,
    pub body:        &'req str,
    #[serde(rename = "tagList")]
    pub tag_list:    Option<Vec<Tag<'req>>>,
} impl CreateArticleRequest<'_> {
    pub fn slug(&self) -> String {
        self.title.chars().filter_map(|ch| match ch {
            '/' | '?' | '=' | '&' | '#'     => None,
            ' ' | '　' | '\r' | '\n' | '\t' => Some('-'),
            _ => Some(ch)
        }).collect()
    }
}

#[Payload(JSOND)]
pub struct UpdateArticleRequest<'req> {
    pub title:       Option<&'req str>,
    pub description: Option<&'req str>,
    pub body:        Option<&'req str>,
}

#[Payload(JSOND)]
pub struct AddCommentRequest<'req> {
    pub content: &'req str,
}
