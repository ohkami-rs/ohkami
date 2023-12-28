use crate::{
    layer0_lib::{CORS, Method},
    IntoFang, Fang, Context, Request, Response,
};

#[allow(non_snake_case)]
pub fn CORS(AllowOrigin: &'static str) -> CORS {
    CORS::new(AllowOrigin)
}

impl IntoFang for CORS {
    fn bite(self) -> Fang {
        #[cold] fn __forbid_cors(c: &Context) -> Result<(), Response> {
            Err(c.Forbidden())
        }

        Fang(move |c: &mut Context, req: &mut Request| -> Result<(), Response> {
            let origin = req.headers.Origin().ok_or_else(|| c.BadRequest())?;
            if self.AllowOrigin.matches(origin) {
                return __forbid_cors(c)
            }

            if req.headers.Authorization().is_some() {
                if !self.AllowCredentials {
                    return __forbid_cors(c)
                }
            }

            if let Some(request_method) = req.headers.AccessControlRequestMethod() {
                let request_method = Method::from_bytes(request_method.as_bytes());
                let allow_methods  = self.AllowMethods.as_ref().ok_or_else(|| c.Forbidden())?;
                if !allow_methods.contains(&request_method) {
                    return __forbid_cors(c)
                }
            }

            if let Some(request_headers) = req.headers.AccessControlRequestHeaders() {
                let request_headers = request_headers.split(',').map(|h| h.trim());
                let allow_headers   = self.AllowHeaders.as_ref().ok_or_else(|| c.Forbidden())?;
                if !request_headers.into_iter().all(|h| allow_headers.contains(&h)) {
                    return __forbid_cors(c)
                }
            }

            c.set_headers().Vary("Origin");
            Ok(())
        })
    }
}
