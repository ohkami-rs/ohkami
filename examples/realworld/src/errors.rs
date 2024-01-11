use ohkami::{IntoResponse, utils::{Text, JSON}};
use std::borrow::Cow;


#[derive(Debug)]
pub enum RealWorldError {
    Config(String),
    DB(sqlx::Error),
    Validation(ValidationError),
    NotFound(Cow<'static, str>),
    FoundUnexpectedly(Cow<'static, str>),
} const _: () = {
    impl std::fmt::Display for RealWorldError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{self:?}"))
        }
    }
    impl std::error::Error for RealWorldError {}
};

#[derive(serde::Serialize)]
struct ValidationErrorFormat {
    errors: ValidationError,
}
#[derive(serde::Serialize)]
#[derive(Debug)]
pub struct ValidationError {
    body: Option<Cow<'static, str>>,
}

impl IntoResponse for RealWorldError {
    fn into_response(self) -> ohkami::Response {
        match self {
            Self::Validation(e) => JSON::UnprocessableEntity(
                ValidationErrorFormat {
                    errors: e,
                }
            ).into(),
            Self::Config(err)           => Text::InternalServerError(err).into(),
            Self::DB(sqlx_err)          => Text::InternalServerError(sqlx_err.to_string()).into(),
            Self::NotFound(nf)          => Text::NotFound(<Cow<'static, str> as Into<String>>::into(nf)).into(),
            Self::FoundUnexpectedly(fu) => Text::BadRequest(<Cow<'static, str> as Into<String>>::into(fu)).into(),
        }
    }
}
