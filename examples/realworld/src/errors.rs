use ohkami::{IntoResponse, utils::Text};


enum RealWorldError {
    DB(sqlx::Error),
}

impl IntoResponse for RealWorldError {
    fn into_response(self) -> ohkami::Response {
        match self {
            Self::DB(sqlx_err) => Text::InternalServerError(sqlx_err.to_string()).into_response(),
        }
    }
}
