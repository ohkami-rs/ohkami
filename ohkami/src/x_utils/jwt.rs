#![allow(non_snake_case)]


pub fn JWT(secret: impl Into<String>) -> internal::JWT {
    #[cfg(test)] {
        const fn assert_into_fang<T: crate::IntoFang>() {}
        assert_into_fang::<internal::JWT>();
    }

    internal::JWT::new(secret)
}

mod internal {
    use crate::layer0_lib::base64_decode;
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
            struct JWTHeader {
                cty: ,
            }

            Fang(|c: &Context, req: &Request| {
                let mut parts = req
                    .headers.Authorization().ok_or_else(|| c.Unauthorized())?
                    .strip_prefix("Bearer ").ok_or_else(|| c.BadRequest())?
                    .split('.');

                let header = base64_decode(parts.next().ok_or_else(|| c.BadRequest())?);

                Ok(())
            })
        }
    }
}
