use std::sync::OnceLock;
use crate::{
    layer0_lib::CORS,
    Context, Request, IntoFang,
};

#[allow(non_snake_case)]
pub fn cors(AllowOrigin: &'static str) -> CORS {
    CORS::new(AllowOrigin)
}

pub(crate) static CORS: OnceLock<(&'static str, CORS)> = OnceLock::new();

impl IntoFang for CORS {
    fn bite(self) -> crate::Fang {
        CORS.set((Box::leak(self.to_string().into_boxed_str()), self)).ok();

        crate::Fang(|c: &mut Context, _: &mut Request| {
            let (cors_str, _) = CORS.get().unwrap();
            c.headers
                .Vary("Origin")
                .cors(cors_str);
        })
    }
}
