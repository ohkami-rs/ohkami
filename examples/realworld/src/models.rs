use ohkami::serde::{Serialize, Deserialize};
use ohkami::fang::JWTToken;
use chrono::{DateTime, Utc};

pub mod request;
pub mod response;


mod serde_datetime {
    use ohkami::serde::{Deserialize, Deserializer, Serializer};
    use chrono::{DateTime, Utc, SecondsFormat};

    pub(super) fn serialize<S: Serializer>(
        date_time:  &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&date_time.to_rfc3339_opts(SecondsFormat::Millis, true))
    }

    #[allow(unused)] // used in test
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let s = String::deserialize(deserializer)?;
        let datetime = DateTime::parse_from_rfc3339(&s)
            .map_err(ohkami::serde::de::Error::custom)?;
        Ok(datetime.into())
    }
}


#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct User {
    pub email: String,
    #[serde(rename = "token")]
    pub jwt:   JWTToken,
    #[serde(rename = "username")]
    pub name:  String,
    pub bio:   Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct Profile {
    pub username:  String,
    pub bio:       Option<String>,
    pub image:     Option<String>,
    pub following: bool,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct Article {
    pub title:           String,
    pub slug:            String,
    pub description:     String,
    pub body:            String,
    #[serde(rename = "tagList")]
    pub tag_list:        Vec<String>,
    #[serde(rename = "createdAt", with = "serde_datetime")]
    pub created_at:      DateTime<Utc>,
    #[serde(rename = "updatedAt", with = "serde_datetime")]
    pub updated_at:      DateTime<Utc>,
    pub favorited:       bool,
    #[serde(rename = "favoritesCount")]
    pub favorites_count: usize,
    pub author:          Profile,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(ohkami::serde::Deserialize, Debug, PartialEq))]
pub struct Comment {
    pub id:         usize,
    #[serde(rename = "createdAt", with = "serde_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt", with = "serde_datetime")]
    pub updated_at: DateTime<Utc>,
    pub body:       String,
    pub author:     Profile,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
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
