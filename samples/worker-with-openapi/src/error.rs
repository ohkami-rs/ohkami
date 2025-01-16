#[derive(Debug, thiserror::Error)]
pub(crate) enum APIError {
    #[error("Error in worker: {0}")]
    Worker(#[from] ::worker::Error),

    #[error("User name `{0}` is already used")]
    UserNameAlreadyUsed(String),
}

impl ohkami::IntoResponse for APIError {
    fn into_response(self) -> ohkami::Response {
        ::worker::console_error!("{self}");
        match &self {
            Self::Worker(_) => ohkami::Response::InternalServerError(),
            Self::UserNameAlreadyUsed(_) => ohkami::Response::BadRequest()  
                .with_text(self.to_string())
        }
    }
}
