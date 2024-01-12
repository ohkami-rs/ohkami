use ohkami::utils::{ResponseBody, Serialize, Deserialize, Serializer};
use chrono::{DateTime, Utc, SecondsFormat};
use uuid::Uuid;

fn serialize_datetime<S: Serializer>(
    date_time:  &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date_time.to_rfc3339_opts(SecondsFormat::Millis, true))
}

#[ResponseBody(JSONS)]
pub struct UserResponse {
    pub user: User,
}
#[derive(Serialize)]
pub struct User {
    pub email:    String,
    /// JWT token
    pub token:    String,
    pub username: String,
    pub bio:      Option<String>,
    pub image:    Option<String>,
}

#[ResponseBody(JSONS)]
pub struct ProfileResponse {
    pub profile: Profile,
}
#[derive(Serialize)]
pub struct Profile {
    pub username:  String,
    pub bio:       String,
    pub image:     String,
    pub following: bool,
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
#[derive(Serialize)]
pub struct Article {
    pub slug:           String,
    pub title:          String,
    pub description:    String,
    pub body:           String,
    #[serde(rename = "tagList")]
    pub tag_list:       Vec<String>,
    #[serde(rename = "createdAt", serialize_with = "serialize_datetime")]
    pub created_at:     DateTime<Utc>,
    #[serde(rename = "updatedAt", serialize_with = "serialize_datetime")]
    pub updated_at:     DateTime<Utc>,
    pub favorited:      bool,
    #[serde(rename = "favoriteCount")]
    pub favorite_count: usize,
    pub author:         Profile,
}

#[ResponseBody(JSONS)]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[ResponseBody(JSONS)]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}
#[derive(Serialize)]
pub struct Comment {
    pub id:         Uuid,
    #[serde(rename = "createdAt", serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
    pub body:       String,
    pub author:     Profile,
}

#[ResponseBody(JSONS)]
pub struct ListOfTagsResponse<'t> {
    pub tags: Vec<Tag<'t>>
}
#[derive(Serialize, Deserialize)]
pub struct Tag<'t>(std::borrow::Cow<'t, str>);
const _: () = {
    impl Tag<'static> {
        pub fn new(name: impl Into<std::borrow::Cow<'static, str>>) -> Self {
            Self(name.into())
        }
    }
    
    impl<'t> std::ops::Deref for Tag<'t> {
        type Target = str;
        #[inline] fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
};
