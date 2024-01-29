use ohkami::utils::{Serialize, Serializer, Deserialize};
use chrono::{DateTime, Utc, SecondsFormat};
use uuid::Uuid;

pub mod request;
pub mod response;


fn serialize_datetime<S: Serializer>(
    date_time:  &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date_time.to_rfc3339_opts(SecondsFormat::Millis, true))
}


#[derive(Serialize)]
pub struct User {
    pub email: String,
    #[serde(rename = "token")]
    pub jwt:   String,
    #[serde(rename = "username")]
    pub name:  String,
    pub bio:   Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize)]
pub struct Profile {
    pub username:  String,
    pub bio:       Option<String>,
    pub image:     Option<String>,
    pub following: bool,
}

#[derive(Serialize)]
pub struct Article {
    pub title:           String,
    pub slug:            Option<String>,
    pub description:     Option<String>,
    pub body:            String,
    #[serde(rename = "tagList")]
    pub tag_list:        Vec<String>,
    #[serde(rename = "createdAt", serialize_with = "serialize_datetime")]
    pub created_at:      DateTime<Utc>,
    #[serde(rename = "updatedAt", serialize_with = "serialize_datetime")]
    pub updated_at:      DateTime<Utc>,
    pub favorited:       bool,
    #[serde(rename = "favoritesCount")]
    pub favorites_count: usize,
    pub author:          Profile,
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
