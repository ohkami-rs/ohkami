#![allow(non_snake_case, non_camel_case_types)]

pub use internal::JWT;

pub fn JWT(secret: impl Into<String>) -> internal::JWT {
    internal::JWT::new(secret)
}

mod internal {
    use crate::layer0_lib::{base64, HMAC_SHA256};
    use crate::{Context, Request, Response};


    pub struct JWT {
        secret: String,
    }
    impl JWT {
        pub fn new(secret: impl Into<String>) -> Self {
            Self {
                secret: secret.into(),
            }
        }
    }

    impl JWT {
        pub fn issue(self, payload: impl ::serde::Serialize) -> String {
            let unsigned_token = {
                let mut ut = base64::encode_url("{\"typ\":\"JWT\",\"alg\":\"HS256\"}");
                ut.push('.');
                ut.push_str(&base64::encode_url(::serde_json::to_vec(&payload).expect("Failed to serialze payload")));
                ut
            };

            let signature = {
                let mut s = HMAC_SHA256::new(self.secret);
                s.write(unsigned_token.as_bytes());
                s.sum()
            };
            
            let mut token = unsigned_token;
            token.push('.');
            token.push_str(&base64::encode_url(signature));
            token
        }
    }

    impl JWT {
        /// Verify JWT in requests' `Authorization` header and early return error response if
        /// it's missing or malformed.
        pub fn verify(&self, c: &Context, req: &Request) -> Result<(), Response> {
            self.verified::<()>(c, req)
        }

        /// Verify JWT in requests' `Authorization` header and early return error response if
        /// it's missing or malformed.
        /// 
        /// Then it's valid, this returns decoded paylaod of the JWT as `Payload`.
        pub fn verified<Payload: for<'d> serde::Deserialize<'d>>(&self, c: &Context, req: &Request) -> Result<Payload, Response> {
            const UNAUTHORIZED_MESSAGE: &str = "missing or malformed jwt";

            type Header  = ::serde_json::Value;
            type Payload = ::serde_json::Value;

            let mut parts = req
                .headers.Authorization().ok_or_else(|| c.Unauthorized().text(UNAUTHORIZED_MESSAGE))?
                .strip_prefix("Bearer ").ok_or_else(|| c.BadRequest())?
                .split('.');

            let header_part = parts.next()
                .ok_or_else(|| c.BadRequest())?;
            let header: Header = ::serde_json::from_slice(&base64::decode_url(header_part))
                .map_err(|_| c.InternalServerError())?;
            if header.get("typ").is_some_and(|typ| !typ.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
                return Err(c.BadRequest())
            }
            if header.get("cty").is_some_and(|cty| !cty.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
                return Err(c.BadRequest())
            }
            if header.get("alg").ok_or_else(|| c.BadRequest())? != "HS256" {
                return Err(c.BadRequest())
            }

            let payload_part = parts.next()
                .ok_or_else(|| c.BadRequest())?;
            let payload: Payload = ::serde_json::from_slice(&base64::decode_url(payload_part))
                .map_err(|_| c.InternalServerError())?;
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            if payload.get("nbf").is_some_and(|nbf| nbf.as_u64().unwrap_or_default() > now) {
                return Err(c.Unauthorized().text(UNAUTHORIZED_MESSAGE))
            }
            if payload.get("exp").is_some_and(|exp| exp.as_u64().unwrap_or_default() <= now) {
                return Err(c.Unauthorized().text(UNAUTHORIZED_MESSAGE))
            }
            if payload.get("iat").is_some_and(|iat| iat.as_u64().unwrap_or_default() > now) {
                return Err(c.Unauthorized().text(UNAUTHORIZED_MESSAGE))
            }

            let signature_part = parts.next()
                .ok_or_else(|| c.BadRequest())?;
            let requested_signature = base64::decode_url(signature_part);
            let actual_signature = {
                let mut hs = HMAC_SHA256::new(&self.secret);
                hs.write(header_part.as_bytes());
                hs.write(b".");
                hs.write(payload_part.as_bytes());
                hs.sum()
            };
            if requested_signature != actual_signature {
                return Err(c.Unauthorized().text(UNAUTHORIZED_MESSAGE))
            }

            let payload = ::serde_json::from_value(payload).map_err(|_| c.InternalServerError())?;
            Ok(payload)
        }
    }
}




#[cfg(test)] mod test {
    use serde::Deserialize;

    use super::JWT;
    use crate::__rt__::test;

    #[test] async fn test_jwt_issue() {
        /* NOTE: 
            `serde_json::to_vec` automatically sorts original object's keys
            in alphabetical order. e.t., here

            ```
            json!({"name":"kanarus","id":42,"iat":1516239022})
            ```
            is serialzed to

            ```raw literal
            {"iat":1516239022,"id":42,"name":"kanarus"}
            ```
        */
        assert_eq! {
            JWT("secret").issue(::serde_json::json!({"name":"kanarus","id":42,"iat":1516239022})),
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE1MTYyMzkwMjIsImlkIjo0MiwibmFtZSI6ImthbmFydXMifQ.dt43rLwmy4_GA_84LMC1m5CwVc59P9as_nRFldVCH7g"
        }
    }

    #[test] async fn test_jwt_verify() {
        use crate::prelude::*;
        use crate::{testing::*, http::*};

        use std::{sync::OnceLock, collections::HashMap, borrow::Cow};
        use crate::__rt__::Mutex;


        fn my_jwt() -> JWT {
            JWT("myverysecretjwtsecretkey")
        }

        #[derive(serde::Serialize, Deserialize)]
        struct MyJWTPayload {
            iat:     u64,
            user_id: usize,
        }

        fn issue_jwt_for_user(user: &User) -> String {
            use std::time::{UNIX_EPOCH, SystemTime};

            my_jwt().issue(MyJWTPayload {
                user_id: user.id,
                iat:     SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            })
        }


        async fn repository() -> &'static Mutex<HashMap<usize, User>> {
            static REPOSITORY: OnceLock<Mutex<HashMap<usize, User>>> = OnceLock::new();

            REPOSITORY.get_or_init(|| Mutex::new(HashMap::new()))
        }

        #[derive(Clone)]
        #[derive(Debug, PartialEq) /* for test */]
        struct User {
            id:           usize,
            first_name:   String,
            familly_name: String,
        } impl User {
            fn profile(&self) -> Profile {
                Profile {
                    id:           self.id,
                    first_name:   &self.first_name,
                    familly_name: &self.familly_name,
                }
            }
        }


        #[derive(serde::Serialize, Deserialize, Debug, PartialEq)]
        struct Profile<'p> {
            id:           usize,
            first_name:   &'p str,
            familly_name: &'p str,
        }

        #[cfg(feature="nightly")]
        async fn get_profile(c: Context) -> Response {
            let r = &mut *repository().await.lock().await;

            let jwt_payload = c.get::<MyJWTPayload>()
                .ok_or_else(|| c.InternalServerError())?;

            let user = r.get(&jwt_payload.user_id)
                .ok_or_else(|| c.BadRequest().text("User doesn't exist"))?;

            c.OK().json(user.profile())
        }
        #[cfg(not(feature="nightly"))]
        async fn get_profile(c: Context) -> Response {
            let r = &mut *repository().await.lock().await;

            let Some(jwt_payload) = c.get::<MyJWTPayload>() else {
                return (|| c.InternalServerError())()
            };

            let Some(user) = r.get(&jwt_payload.user_id) else {
                return (|| c.BadRequest().text("User doesn't exist"))()
            };

            c.OK().json(user.profile())
        }

        #[derive(serde::Deserialize, serde::Serialize/* for test */)]
        struct SigninRequest<'s> {
            first_name:   &'s str,
            familly_name: &'s str,
        } impl<'req> crate::FromRequest<'req> for SigninRequest<'req> {
            type Error = std::borrow::Cow<'static, str>;
            fn parse(req: &'req Request) -> Result<Self, Self::Error> {
                serde_json::from_slice(
                    req.payload().ok_or_else(|| std::borrow::Cow::Borrowed("No payload found"))?
                ).map_err(|e| std::borrow::Cow::Owned(e.to_string()))
            }
        }

        async fn signin(c: Context, body: SigninRequest<'_>) -> Response {
            let r = &mut *repository().await.lock().await;

            let user: Cow<'_, User> = match r.iter().find(|(_, u)|
                u.first_name   == body.first_name &&
                u.familly_name == body.familly_name
            ) {
                Some((_, u)) => Cow::Borrowed(u),
                None => {
                    let new_user_id = match r.keys().max() {
                        Some(max) => max + 1,
                        None      => 1,
                    };

                    let new_user = User {
                        id:           new_user_id,
                        first_name:   body.first_name.to_string(),
                        familly_name: body.familly_name.to_string(), 
                    };

                    r.insert(new_user_id, new_user.clone());

                    Cow::Owned(new_user)
                }
            };

            c.OK().text(issue_jwt_for_user(&user))
        }


        struct MyJWTFang(JWT);
        impl IntoFang for MyJWTFang {
            fn into_fang(self) -> Fang {
                Fang(move |c: &mut Context, req: &Request| {
                    let jwt_payload =  self.0.verified::<MyJWTPayload>(c, req)?;
                    c.store(jwt_payload);
                    Ok(())
                })
            }
        }

        let t = Ohkami::new((
            "/signin".By(Ohkami::new(
                "/".PUT(signin),
            )),
            "/profile".By(Ohkami::with((
                MyJWTFang(my_jwt()),
            ), (
                "/".GET(get_profile),
            ))),
        ));
        

        let req = TestRequest::PUT("/signin");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::BadRequest);

        let req = TestRequest::GET("/profile");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(res.text(),   Some("missing or malformed jwt"));


        let req = TestRequest::PUT("/signin")
            .json(SigninRequest {
                first_name:   "ohkami",
                familly_name: "framework",
            });
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        let jwt_1 = dbg!(res.text().unwrap());

        let req = TestRequest::GET("/profile")
            .header("Authorization", format!("Bearer {jwt_1}"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.json::<Profile>().unwrap().unwrap(), Profile {
            id:           1,
            first_name:   "ohkami",
            familly_name: "framework",
        });

        let req = TestRequest::GET("/profile")
            .header("Authorization", format!("Bearer {jwt_1}x"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(res.text(),   Some("missing or malformed jwt"));


        assert_eq! {
            &*repository().await.lock().await,
            &HashMap::from([
                (1, User {
                    id:           1,
                    first_name:   format!("ohkami"),
                    familly_name: format!("framework"),
                }),
            ])
        }


        let req = TestRequest::PUT("/signin")
            .json(SigninRequest {
                first_name:   "Leonhard",
                familly_name: "Euler",
            });
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        let jwt_2 = dbg!(res.text().unwrap());

        let req = TestRequest::GET("/profile")
            .header("Authorization", format!("Bearer {jwt_2}"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.json::<Profile>().unwrap().unwrap(), Profile {
            id:           2,
            first_name:   "Leonhard",
            familly_name: "Euler",
        });


        assert_eq! {
            &*repository().await.lock().await,
            &HashMap::from([
                (1, User {
                    id:           1,
                    first_name:   format!("ohkami"),
                    familly_name: format!("framework"),
                }),
                (2, User {
                    id:           2,
                    first_name:   format!("Leonhard"),
                    familly_name: format!("Euler"),
                }),
            ])
        }


        let req = TestRequest::GET("/profile")
            .header("Authorization", format!("Bearer {jwt_1}"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.json::<Profile>().unwrap().unwrap(), Profile {
            id:           1,
            first_name:   "ohkami",
            familly_name: "framework",
        });

        let req = TestRequest::GET("/profile")
            .header("Authorization", format!("Bearer {jwt_2}0000"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!(res.text(),   Some("missing or malformed jwt"));


        assert_eq! {
            &*repository().await.lock().await,
            &HashMap::from([
                (1, User {
                    id:           1,
                    first_name:   format!("ohkami"),
                    familly_name: format!("framework"),
                }),
                (2, User {
                    id:           2,
                    first_name:   format!("Leonhard"),
                    familly_name: format!("Euler"),
                }),
            ])
        }
    }
}
