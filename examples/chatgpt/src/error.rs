use ohkami::prelude::*;

#[derive(Debug)]
pub enum Error {
    Fetch(reqwest::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("{self}");
        match self {
            Self::Fetch(_) => Response::InternalServerError(),
        }
    }
}

const _: () = {
    impl std::error::Error for Error {}
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Debug::fmt(self, f)
        }
    }

    impl From<reqwest::Error> for Error {
        fn from(e: reqwest::Error) -> Self {
            Self::Fetch(e)
        }
    }
};
