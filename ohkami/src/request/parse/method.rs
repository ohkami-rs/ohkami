use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PATCH,
    DELETE,
}

impl Method {
    pub(crate) fn parse(string: &str) -> Self {
        match string {
            "GET"    => Self::GET,
            "POST"   => Self::POST,
            "PATCH"  => Self::PATCH,
            "DELETE" => Self::DELETE,
            _ => {
                tracing::error!("unknown method: {string}");
                panic!()
            },
        }
    }
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::GET    => 3,
            Self::POST   => 4,
            Self::PATCH  => 5,
            Self::DELETE => 6,
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::GET    => "GET",
            Self::POST   => "POST",
            Self::PATCH  => "PATCH",
            Self::DELETE => "DELETE",
        })
    }
}