use crate::{Response, Status};


#[allow(non_snake_case)]
pub fn cors(AllowOrigin: &'static str) -> crate::cors::CORS {
    crate::cors::CORS::new(AllowOrigin)
}
impl super::IntoFang<crate::cors::CORS> for crate::cors::CORS {
    fn into_fang(self) -> Option<super::Fang> {
        crate::setCORS(self);
        None
    }
}


pub fn not_found(proc: impl Fn(Response)->Response) -> impl Fn(Response)->Response {
    move |res| {
        match res.status {
            Status::NotFound => proc(res),
            _  => res
        }
    }
}
