use ohkami::serde::{Serialize, Deserialize};

#[cfg(feature="openapi")]
use ohkami::openapi::Schema;


pub(super) type ID = i32;

pub(super) type Age = u8;

pub(super) type Timestamp = String;

pub(super) fn timestamp_now() -> Timestamp {
    ohkami::util::unix_timestamp().to_string()
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
#[cfg_attr(feature="openapi", openapi(component))]
pub(super) struct UserProfile {
    pub(super) id:       ID,
    pub(super) name:     String,
    pub(super) location: Option<String>,
    pub(super) age:      Option<Age>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
pub(super) struct EditProfileRequest<'req> {
    pub(super) location: Option<&'req str>,
    pub(super) age:      Option<Age>,
}

#[derive(Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
pub(super) struct SignUpRequest<'req> {
    pub(super) name:  &'req str,
    pub(super) token: &'req str,
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature="openapi", derive(Schema))]
#[cfg_attr(feature="openapi", openapi(component))]
pub(super) struct Tweet {
    pub(super) user_id:   ID,
    pub(super) user_name: String,
    pub(super) content:   String,
    pub(super) posted_at: Timestamp,
}

#[derive(Deserialize)]
#[cfg_attr(feature="openapi", derive(Schema))]
pub(super) struct PostTweetRequest<'req> {
    pub(super) content: &'req str,
}

