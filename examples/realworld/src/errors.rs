use ohkami::{IntoResponse, utils::Text};


#[derive(Debug)]
pub enum RealWorldError {
    Config(&'static str),
    DB(sqlx::Error),
}

impl IntoResponse for RealWorldError {
    fn into_response(self) -> ohkami::Response {
        match self {
            Self::Config(err)  => Text::InternalServerError(err).into_response(),
            Self::DB(sqlx_err) => Text::InternalServerError(sqlx_err.to_string()).into_response(),
        }
    }
}
