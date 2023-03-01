mod path;
mod query;
mod body;

pub trait FromRequest {
    fn from_request<'buf>(request: &super::Request) -> Self;
}
