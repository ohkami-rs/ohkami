pub mod kv;

use crate::FromRequest;


#[allow(non_snake_case)]
fn AssertSend<T: Send>() {}

pub struct Bindings<'worker>(&'worker worker::Env);
impl<'req> FromRequest<'req> for Bindings<'req> {
    type Error = std::convert::Infallible;
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        Some(Ok(Self(req.env())))
    }
}
#[allow(non_snake_case)]
impl<'worker> Bindings<'worker> {
    pub fn KV(&self, name: &'static str) -> Result<kv::KV, worker::Error> {
        self.0.kv(name).map(kv::KV)
    }
}
