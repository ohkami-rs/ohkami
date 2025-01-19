use crate::model::ID;

#[derive(Debug, thiserror::Error)]
pub(crate) enum APIError {
    #[error("Error in worker: {0}")]
    Worker(#[from] ::worker::Error),

    #[error("User name `{0}` is already used")]
    UserNameAlreadyUsed(String),

    #[error("User (id = {id}) not found")]
    UserNotFound { id: ID },

    #[error("User (id = {me}) requests modifying other user (id = {other})")]
    ModifyingOtherUser { me: ID, other: ID },
}

impl ohkami::IntoResponse for APIError {
    fn into_response(self) -> ohkami::Response {
        ::worker::console_error!("{self}");
        match &self {
            Self::Worker(_) => ohkami::Response::InternalServerError(),
            Self::UserNameAlreadyUsed(_) => ohkami::Response::BadRequest()  
                .with_text(self.to_string()),
            Self::UserNotFound { .. } => ohkami::Response::NotFound(),
            Self::ModifyingOtherUser { .. } => ohkami::Response::Forbidden(),
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_responses() -> ohkami::openapi::Responses {
        use ohkami::openapi::{self, Response};

        ohkami::openapi::Responses::enumerated([
            (500, Response::when("Worker's internal error")),
            (400, Response::when("Username already used")
                .content("text/plain", openapi::string())),
            (404, Response::when("User not found")),
            (403, Response::when("Modyfing other user"))
        ])
    }
}
