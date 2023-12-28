use crate::{
    layer0_lib::CORS,
    IntoFang,
    Context, Request,
};

#[allow(non_snake_case)]
pub fn CORS(AllowOrigin: &'static str) -> CORS {
    CORS::new(AllowOrigin)
}

impl IntoFang for CORS {
    fn bite(self) -> crate::Fang {
        crate::Fang(|c: &mut Context, req: &mut Request| {
            // let (cors_str, _) = CORS.get().unwrap();
            // c.set_headers()
            //     .Vary("Origin")
            //     .cors(cors_str);
        })
    }
}

/*


                {
                    let Some(origin) = req.header("Origin") else {
                        return __no_upgrade(c.BadRequest());
                    };
                    if !cors.AllowOrigin.matches(origin) {
                        return __no_upgrade(c.Forbidden());
                    }

                    if req.header("Authorization").is_some() && !cors.AllowCredentials {
                        return __no_upgrade(c.Forbidden());
                    }

                    if let Some(request_method) = req.header("Access-Control-Request-Method") {
                        let request_method = Method::from_bytes(request_method.as_bytes());
                        let Some(allow_methods) = cors.AllowMethods.as_ref() else {
                            return __no_upgrade(c.Forbidden());
                        };
                        if !allow_methods.contains(&request_method) {
                            return __no_upgrade(c.Forbidden());
                        }
                    }

                    if let Some(request_headers) = req.header("Access-Control-Request-Headers") {
                        let mut request_headers = request_headers.split(',').map(|h| h.trim_matches(' '));
                        let Some(allow_headers) = cors.AllowHeaders.as_ref() else {
                            return __no_upgrade(c.Forbidden());
                        };
                        if !request_headers.all(|h| allow_headers.contains(&h)) {
                            return __no_upgrade(c.Forbidden());
                        }
                    }
                }


*/