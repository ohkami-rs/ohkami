use ohkami::{IntoResponse, serde::Serialize};
use std::borrow::Cow;

#[derive(Debug)]
pub enum RealWorldError {
    Config(String),
    DB(sqlx::Error),
    Validation { body: String },
    NotFound(Cow<'static, str>),
    Unauthorized(Cow<'static, str>),
    FoundUnexpectedly(Cow<'static, str>),
} const _: () = {
    impl std::fmt::Display for RealWorldError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{self:?}"))
        }
    }
    impl std::error::Error for RealWorldError {}
};

#[derive(Serialize)]
struct ValidationErrorFormat {
    errors: ValidationError,
}
#[derive(Serialize, Debug)]
pub struct ValidationError {
    body: Vec<Cow<'static, str>>,
}

impl IntoResponse for RealWorldError {
    fn into_response(self) -> ohkami::Response {
        use ohkami::claw::{status, Json};
        
        match self {
            Self::Validation { body } => status::UnprocessableEntity(
                Json(ValidationErrorFormat {
                    errors: ValidationError {
                        body: vec![body.into()],
                    },
                }
            )).into_response(),
            Self::Config(err_msg)       => status::InternalServerError(err_msg).into_response(),
            Self::DB(sqlx_err)          => status::InternalServerError(sqlx_err.to_string()).into_response(),
            Self::NotFound(nf)          => status::NotFound(nf).into_response(),
            Self::Unauthorized(msg)     => status::Unauthorized(msg).into_response(),
            Self::FoundUnexpectedly(fu) => status::BadRequest(fu).into_response(),
        }
    }
}
