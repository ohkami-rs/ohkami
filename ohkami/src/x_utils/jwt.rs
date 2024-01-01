#![allow(non_snake_case)]


pub fn JWT(secret: impl Into<String>) -> internal::JWT {
    #[cfg(test)] {
        const fn assert_into_fang<T: crate::IntoFang>() {}
        assert_into_fang::<internal::JWT>();
    }

    internal::JWT::new(secret)
}

mod internal {
    use crate::layer0_lib::{base64, HMACSha256};
    use crate::{IntoFang, Fang, Context, Request};


    pub struct JWT {
        secret: String,
        alg:    VerifyingAlgorithm,
    }
    impl JWT {
        pub fn new(secret: impl Into<String>) -> Self {
            Self {
                secret: secret.into(),
                alg:    VerifyingAlgorithm::default(),
            }
        }
    }

    macro_rules! VerifyingAlgorithm {
        { $( $alg:ident, )+ @default: $default:ident } => {
            enum VerifyingAlgorithm {
                $(
                    $alg,
                )*
            }
            impl Default for VerifyingAlgorithm {
                fn default() -> Self {
                    VerifyingAlgorithm::$default
                }
            }
            impl VerifyingAlgorithm {
                const fn as_str(&self) -> &'static str {
                    match self {
                        $(
                            Self::$alg => stringify!($alg),
                        )*
                    }
                }
            }

            impl JWT {
                $(
                    pub fn $alg(mut self) -> Self {
                        self.alg = VerifyingAlgorithm::$alg;
                        self
                    }
                )*
            }
        };
    } VerifyingAlgorithm! {
        HS256,
        HS384,
        HS512,

        @default: HS256
    }


    impl IntoFang for JWT {
        fn into_fang(self) -> Fang {
            type Header  = ::serde_json::Value;
            type Payload = ::serde_json::Value;
            
            Fang(move |c: &Context, req: &Request| {
                let mut parts = req
                    .headers.Authorization().ok_or_else(|| c.Unauthorized())?
                    .strip_prefix("Bearer ").ok_or_else(|| c.BadRequest())?
                    .split('.');

                let header_part = parts.next()
                    .ok_or_else(|| c.BadRequest())?;
                let header: Header = ::serde_json::from_slice(&base64::decode(header_part))
                    .map_err(|_| c.InternalServerError())?;
                if header.get("typ").is_some_and(|typ| typ.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
                    return Err(c.BadRequest())
                }
                if header.get("cty").is_some_and(|cty| cty.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
                    return Err(c.BadRequest())
                }
                if header.get("alg").ok_or_else(|| c.BadRequest())? != self.alg.as_str() {
                    return Err(c.BadRequest())
                }

                let payload_part = parts.next()
                    .ok_or_else(|| c.BadRequest())?;
                let payload: Payload = ::serde_json::from_slice(&base64::decode(payload_part))
                    .map_err(|_| c.InternalServerError())?;
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                if payload.get("nbf").is_some_and(|nbf| nbf.as_u64().unwrap_or_default() > now) {
                    return Err(c.BadRequest())
                }
                if payload.get("exp").is_some_and(|exp| exp.as_u64().unwrap_or_default() <= now) {
                    return Err(c.BadRequest())
                }
                if payload.get("iat").is_some_and(|iat| iat.as_u64().unwrap_or_default() > now) {
                    return Err(c.BadRequest())
                }

                let signature_part = parts.next()
                    .ok_or_else(|| c.BadRequest())?;
                let requested_signature = base64::decode_url(signature_part);
                let actual_signature = {
                    let mut s = HMACSha256::new();
                    s.write(header_part.as_bytes());
                    s.write(b".");
                    s.write(payload_part.as_bytes());
                    s.sum()
                };
                if requested_signature != actual_signature {
                    return Err(c.Unauthorized().text("mulformed jwt"))
                }

                Ok(())
            })
        }
    }
}
