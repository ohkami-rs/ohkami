use serde::Serialize;
use chrono::{DateTime, Utc, SecondsFormat};

fn serialize_datetime<S: serde::Serializer>(
    date_time:  &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date_time.to_rfc3339_opts(SecondsFormat::Millis, true))
}


#[derive(Serialize)]
pub struct UserResponse {
    pub user: User,
}
#[derive(Serialize)]
pub struct User {
    pub email:    String,
    /// JWT token
    pub token:    String,
    pub username: String,
    pub bio:      String,
    pub image:    Option<String>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct SingleArticleResponse {
    pub article: Article,
}
#[derive(Serialize)]
pub struct MultipleArticleResponse {
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

#[derive(Serialize)]
pub struct SingleCommentResponse {
    pub comment: Comment,
}
#[derive(Serialize)]
pub struct MultipleCommentsResponse {
    pub comments: Vec<Comment>,
}
#[derive(Serialize)]
pub struct Comment {
    pub id:         usize,
    #[serde(rename = "createdAt", serialize_with = "serialize_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", serialize_with = "serialize_datetime")]
    pub updated_at: DateTime<Utc>,
    pub body:       String,
    pub author:     Profile,
}

#[derive(Serialize)]
pub struct ListOfTagsResponse {
    pub tags: Vec<Tag>
}
#[derive(Serialize)]
pub struct Tag(pub String);
