#![allow(non_snake_case)]


pub fn JWT() -> internal::JWT {
    #[cfg(test)] {
        const fn assert_into_fang<T: crate::IntoFang>() {}
        assert_into_fang::<internal::JWT>();
    }

    internal::JWT::default()
}

mod internal {
    use crate::{http::Method, IntoFang, Fang, Context, Response, Request};


    pub struct JWT {
        secret: String,
        alg:    VerifyingAlgorithm,
    }
    impl Default for JWT {
        fn default() -> Self {
            JWT {
                secret: super::super::now().replace(' ', "+"),
                alg:    VerifyingAlgorithm::default(),
            }
        }
    }
    impl JWT {
        pub fn secret(mut self, secret: impl Into<String>) -> Self {
            self.secret = secret.into();
            self
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
            Fang(|c: &Context, req: &Request| {
                let parts = req
                    .headers.Authorization().ok_or_else(|| c.Forbidden())?
                    .strip_prefix("Bearer ").ok_or_else(|| c.BadRequest())?
                    .split('.');

                todo!{}

                Ok(())
            })
        }
    }
}
