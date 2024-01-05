#![allow(non_snake_case)]

pub use internal::CORS;

pub const fn CORS(AllowOrigin: &'static str) -> internal::CORS {
    #[cfg(test)] {
        const fn assert_into_fang<T: crate::IntoFang>() {}
        assert_into_fang::<internal::CORS>();
    }

    internal::CORS {
        AllowOrigin:      internal::AccessControlAllowOrigin::from_literal(AllowOrigin),
        AllowCredentials: false,
        AllowMethods:     None,
        AllowHeaders:     None,
        ExposeHeaders:    None,
        MaxAge:           None,
    }
}

mod internal {
    use crate::{http::Method, IntoFang, Fang, Response, Request, response as res};


    pub struct CORS {
        pub(crate) AllowOrigin:      AccessControlAllowOrigin,
        pub(crate) AllowCredentials: bool,
        pub(crate) AllowMethods:     Option<Vec<Method>>,
        pub(crate) AllowHeaders:     Option<Vec<&'static str>>,
        pub(crate) ExposeHeaders:    Option<Vec<&'static str>>,
        pub(crate) MaxAge:           Option<u32>,
    }

    pub(crate) enum AccessControlAllowOrigin {
        Any,
        Only(&'static str),
    } impl AccessControlAllowOrigin {
        #[inline(always)] pub(crate) const fn is_any(&self) -> bool {
            match self {
                Self::Any => true,
                _ => false,
            }
        }

        #[inline(always)] pub(crate) const fn from_literal(lit: &'static str) -> Self {
            match lit.as_bytes() {
                b"*"   => Self::Any,
                origin => Self::Only(unsafe{std::str::from_utf8_unchecked(origin)}),
            }
        }

        #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
            match self {
                Self::Any          => "*",
                Self::Only(origin) => origin,
            }
        }

        #[inline(always)] pub(crate) fn matches(&self, origin: &str) -> bool {
            match self {
                Self::Any     => true,
                Self::Only(o) => *o == origin,
            }
        }
    }

    impl CORS {
        pub const fn AllowCredentials(mut self) -> Self {
            if self.AllowOrigin.is_any() {
                panic!("\
                    The value of the 'Access-Control-Allow-Origin' header in the response \
                    must not be the wildcard '*' when the request's credentials mode is 'include'.\
                ")
            }
            self.AllowCredentials = true;
            self
        }
        pub fn AllowMethods<const N: usize>(mut self, methods: [Method; N]) -> Self {
            self.AllowMethods = Some(methods.to_vec());
            self
        }
        pub fn AllowHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
            self.AllowHeaders = Some(headers.to_vec());
            self
        }
        pub fn ExposeHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
            self.ExposeHeaders = Some(headers.to_vec());
            self
        }
        pub fn MaxAge(mut self, delta_seconds: u32) -> Self {
            self.MaxAge = Some(delta_seconds);
            self
        }
    }

    impl IntoFang for CORS {
        const METHODS: &'static [Method] = &[Method::OPTIONS];

        fn into_fang(self) -> Fang {
            #[cold] fn __forbid_cors() -> Result<(), Response> {
                Err(res::Empty::Forbidden().into())
            }

            Fang(move |req: &mut Request| -> Result<(), Response> {
                c.set_headers()
                    .AccessControlAllowOrigin(self.AllowOrigin.as_str())
                    .AccessControlAllowCredentials(if self.AllowCredentials {"true"} else {"false"});
                if let Some(methods) = &self.AllowMethods {
                    c.set_headers()
                    .AccessControlAllowMethods(methods.iter().map(Method::as_str).collect::<Vec<_>>().join(","));
                }
                if let Some(headers) = &self.AllowHeaders {
                    c.set_headers()
                    .AccessControlAllowHeaders(headers.join(","));
                }
                if let Some(headers) = &self.ExposeHeaders {
                    c.set_headers()
                    .AccessControlExposeHeaders(headers.join(","));
                }

                let origin = req.headers.Origin().ok_or_else(|| c.BadRequest())?;
                if !self.AllowOrigin.matches(origin) {
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
}
