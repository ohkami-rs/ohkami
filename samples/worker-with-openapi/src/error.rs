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
}
