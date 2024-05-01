#![allow(unused)]
fn main() {}

use ohkami::{FromRequest, Method};


struct RequestMethod(Method);
impl<'req> FromRequest<'req> for RequestMethod {
    type Error = std::convert::Infallible;
    fn from_request(req: &'req ohkami::prelude::Request) -> Option<Result<Self, Self::Error>> {
        Some(Ok(Self(req.method)))
    }
}

struct RequestPath<'req>(std::borrow::Cow<'req, str>);
impl<'req> FromRequest<'req> for RequestPath<'req> {
    type Error = std::convert::Infallible;
    fn from_request(req: &'req ohkami::prelude::Request) -> Option<Result<Self, Self::Error>> {
        Some(Ok(Self(req.path.str())))
    }
}

struct RequestPathOwned(String);
impl<'req> FromRequest<'req> for RequestPathOwned {
    type Error = std::convert::Infallible;
    fn from_request(req: &'req ohkami::prelude::Request) -> Option<Result<Self, Self::Error>> {
        Some(Ok(Self(req.path.str().into())))
    }
}


#[derive(FromRequest)]
struct MethodAndPathA {
    method: RequestMethod,
    path:   RequestPathOwned,
}

#[derive(FromRequest)]
struct MethodAndPathB<'req> {
    method: RequestMethod,
    path:   RequestPath<'req>,
}

#[derive(FromRequest)]
struct MethodAndPathC(
    RequestMethod,
    RequestPathOwned,
);

#[derive(FromRequest)]
struct MethodAndPathD<'req>(
    RequestMethod,
    RequestPath<'req>,
);
