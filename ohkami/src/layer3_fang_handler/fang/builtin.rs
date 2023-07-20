use crate::{Response, Status};


pub fn cors() -> crate::layer1_req_res::CORS {
    crate::layer1_req_res::CORS::new()
}
impl super::IntoFang<crate::layer1_req_res::CORS> for crate::layer1_req_res::CORS {
    fn into_fang(self) -> Option<super::Fang> {
        if let Err(e) = crate::layer1_req_res::headers::CORS.set(self.into_static()) {
            panic!("Can't set CORS: {e}")
        }
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
