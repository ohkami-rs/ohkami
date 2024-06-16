use ohkami::prelude::*;


#[derive(Debug)]
pub enum Error {
    Fetch(reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Fetch(e) => Response::InternalServerError().with_text(e.to_string()),
        }
    }
}

const _: () = {
    impl std::error::Error for Error {}
    
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Fetch(e) => e.fmt(f)
            }
        }
    }
};

const _: () = {
    impl From<reqwest::Error> for Error {
        fn from(e: reqwest::Error) -> Self {
            Self::Fetch(e)
        }
    }
};
