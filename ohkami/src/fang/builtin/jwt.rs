#![allow(non_snake_case, non_camel_case_types)]

use std::borrow::Cow;
use ohkami_lib::base64;
use crate::{Request, Response, Status};


/// # Builtin fang for JWT (JSON Web Token) config
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::{prelude::*, Memory};
/// use ohkami::builtin::JWT;
/// use ohkami::typed::ResponseBody;
/// use ohkami::serde::{Serialize, Deserialize};
/// 
/// 
/// fn my_jwt() -> JWT {
///     let jwt_secret = std::env::var("JWT_SECRET").unwrap();
/// 
///     // `default` uses HMAC-SHA256 as the verifying algorithm
///     JWT::default(jwt_secret)
/// }
/// 
/// #[derive(Serialize, Deserialize)]
/// struct JWTPayload {
///     user_id: i64,
///     iat:     u64,
/// }
/// 
/// struct MyAuthFang;
/// impl IntoFang for MyAuthFang {
///     fn into_fang(self) -> Fang {
///         Fang::front(move |req: &mut Request| {
///             let payload = my_jwt()
///                 .verified::<JWTPayload>(req)?;
///             req.memorize(payload);
///             Ok(())
///         })
///     }
/// }
/// 
/// fn issue_jwt_for(user_id: i64) -> String {
///     my_jwt().issue(JWTPayload {
///         user_id,
///         iat: ohkami::utils::unix_timestamp(),
///     })
/// }
/// 
/// 
/// #[ResponseBody(JSONS)]
/// struct SigninResponse {
///     token: String,
/// }
/// 
/// async fn signin() -> SigninResponse {
///     SigninResponse {
///         token: issue_jwt_for(42)
///     }
/// }
/// 
/// 
/// #[ResponseBody(JSONS)]
/// struct ProfileResponse {
///     name: String,
///     bio:  Option<String>,
/// }
/// 
/// async fn get_profile(
///     auth: Memory<'_, JWTPayload>
/// ) -> ProfileResponse {
///     ProfileResponse {
///         name: String::from("ohkami"),
///         bio:  Some(String::from("declarative web framework")),
///     }
/// }
/// 
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(MyAuthFang, (
///         "/signin".POST(signin),
///         "/api/profile".GET(get_profile),
///     )).howl(3000).await
/// }
/// ```
#[derive(Clone)]
pub struct JWT {
    secret: Cow<'static, str>,
    alg:    VerifyingAlgorithm,
}
#[derive(Clone)]
enum VerifyingAlgorithm {
    HS256,
    HS384,
    HS512,
}

impl JWT {
    /// Just `new_256`; use HMAC-SHA256 as verifying algorithm
    #[inline] pub fn default(secret: impl Into<Cow<'static, str>>) -> Self {
        Self {
            secret: secret.into(),
            alg:    VerifyingAlgorithm::HS256,
        }
    }
    /// Use HMAC-SHA256 as verifying algorithm
    pub fn new_256(secret: impl Into<Cow<'static, str>>) -> Self {
        Self {
            secret: secret.into(),
            alg:    VerifyingAlgorithm::HS256,
        }
    }
    /// Use HMAC-SHA384 as verifying algorithm
    pub fn new_384(secret: impl Into<Cow<'static, str>>) -> Self {
        Self {
            secret: secret.into(),
            alg:    VerifyingAlgorithm::HS384,
        }
    }
    /// Use HMAC-SHA512 as verifying algorithm
    pub fn new_512(secret: impl Into<Cow<'static, str>>) -> Self {
        Self {
            secret: secret.into(),
            alg:    VerifyingAlgorithm::HS512,
        }
    }


    #[inline(always)] const fn alg_str(&self) -> &'static str {
        match self.alg {
            VerifyingAlgorithm::HS256 => "HS256",
            VerifyingAlgorithm::HS384 => "HS384",
            VerifyingAlgorithm::HS512 => "HS512",
        }
    }
    #[inline(always)] const fn header_str(&self) -> &'static str {
        match self.alg {
            VerifyingAlgorithm::HS256 => "{\"typ\":\"JWT\",\"alg\":\"HS256\"}",
            VerifyingAlgorithm::HS384 => "{\"typ\":\"JWT\",\"alg\":\"HS384\"}",
            VerifyingAlgorithm::HS512 => "{\"typ\":\"JWT\",\"alg\":\"HS512\"}",
        }
    }
}

impl JWT {
    #[inline] pub fn issue(self, payload: impl ::serde::Serialize) -> String {
        let unsigned_token = {
            let mut ut = base64::encode_url(self.header_str());
            ut.push('.');
            ut.push_str(&base64::encode_url(::serde_json::to_vec(&payload).expect("Failed to serialze payload")));
            ut
        };

        let signature = {
            use ::sha2::{Sha256};
            use ::hmac::{Hmac, KeyInit, Mac};

            let mut s = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
            s.update(unsigned_token.as_bytes());
            s.finalize().into_bytes()
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
    pub fn verify(&self, req: &Request) -> Result<(), Response> {
        self.verified::<()>(req)
    }

    /// Verify JWT in requests' `Authorization` header and early return error response if
    /// it's missing or malformed.
    /// 
    /// Then it's valid, this returns decoded paylaod of the JWT as `Payload`.
    pub fn verified<Payload: for<'d> serde::Deserialize<'d>>(&self, req: &Request) -> Result<Payload, Response> {
        const UNAUTHORIZED_MESSAGE: &str = "missing or malformed jwt";

        type Header  = ::serde_json::Value;
        type Payload = ::serde_json::Value;

        let mut parts = req
            .headers.Authorization().ok_or_else(|| Response::with(Status::Unauthorized).text(UNAUTHORIZED_MESSAGE))?
            .strip_prefix("Bearer ").ok_or_else(|| Response::with(Status::BadRequest))?
            .split('.');

        let header_part = parts.next()
            .ok_or_else(|| Response::with(Status::BadRequest))?;
        let header: Header = ::serde_json::from_slice(&base64::decode_url(header_part))
            .map_err(|_| Response::with(Status::InternalServerError))?;
        if header.get("typ").is_some_and(|typ| !typ.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
            return Err(Response::with(Status::BadRequest))
        }
        if header.get("cty").is_some_and(|cty| !cty.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
            return Err(Response::with(Status::BadRequest))
        }
        if header.get("alg").ok_or_else(|| Response::with(Status::BadRequest))? != self.alg_str() {
            return Err(Response::with(Status::BadRequest))
        }

        let payload_part = parts.next()
            .ok_or_else(|| Response::with(Status::BadRequest))?;
        let payload: Payload = ::serde_json::from_slice(&base64::decode_url(payload_part))
            .map_err(|_| Response::with(Status::InternalServerError))?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        if payload.get("nbf").is_some_and(|nbf| nbf.as_u64().unwrap_or_default() > now) {
            return Err(Response::with(Status::Unauthorized).text(UNAUTHORIZED_MESSAGE))
        }
        if payload.get("exp").is_some_and(|exp| exp.as_u64().unwrap_or_default() <= now) {
            return Err(Response::with(Status::Unauthorized).text(UNAUTHORIZED_MESSAGE))
        }
        if payload.get("iat").is_some_and(|iat| iat.as_u64().unwrap_or_default() > now) {
            return Err(Response::with(Status::Unauthorized).text(UNAUTHORIZED_MESSAGE))
        }

        let signature_part = parts.next().ok_or_else(|| Response::with(Status::BadRequest))?;
        let requested_signature = base64::decode_url(signature_part);

        let is_correct_signature = {
            use ::sha2::{Sha256, Sha384, Sha512};
            use ::hmac::{Hmac, KeyInit, Mac};

            match self.alg {
                VerifyingAlgorithm::HS256 => {
                    let mut hs = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().0 == *requested_signature
                }
                VerifyingAlgorithm::HS384 => {
                    let mut hs = Hmac::<Sha384>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().0 == *requested_signature
                }
                VerifyingAlgorithm::HS512 => {
                    let mut hs = Hmac::<Sha512>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().0 == *requested_signature
                }
            }
        };
        
        if !is_correct_signature {
            return Err(Response::with(Status::Unauthorized).text(UNAUTHORIZED_MESSAGE))
        }

        let payload = ::serde_json::from_value(payload).map_err(|_| Response::with(Status::InternalServerError))?;
        Ok(payload)
    }
}




#[cfg(feature="testing")]
#[cfg(test)] mod test {
    use super::JWT;
    use crate::{__rt__::test, utils};

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
            JWT::default("secret").issue(::serde_json::json!({"name":"kanarus","id":42,"iat":1516239022})),
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE1MTYyMzkwMjIsImlkIjo0MiwibmFtZSI6ImthbmFydXMifQ.dt43rLwmy4_GA_84LMC1m5CwVc59P9as_nRFldVCH7g"
        }
    }

    #[test] async fn test_jwt_verify() {
        use crate::{Request, testing::TestRequest, Status};
        use std::pin::Pin;

        let my_jwt = JWT::default("ohkami-realworld-jwt-authorization-secret-key");

        let req_bytes = TestRequest::GET("/")
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MDY4MTEwNzUsInVzZXJfaWQiOiI5ZmMwMDViMi1mODU4LTQzMzYtODkwYS1mMWEyYWVmNjBhMjQifQ.AKp-0zvKK4Hwa6qCgxskckD04Snf0gpSG7U1LOpcC_I")
            .encode();
        let mut req = Request::init();
        let mut req = unsafe {Pin::new_unchecked(&mut req)};
        req.as_mut().read(&mut &req_bytes[..]).await;

        assert_eq!(
            my_jwt.verified::<::serde_json::Value>(&req.as_ref()).unwrap(),
            ::serde_json::json!({ "iat": 1706811075, "user_id": "9fc005b2-f858-4336-890a-f1a2aef60a24" })
        );

        let req_bytes = TestRequest::GET("/")
            // Modifed last `I` of the value above to `X`
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MDY4MTEwNzUsInVzZXJfaWQiOiI5ZmMwMDViMi1mODU4LTQzMzYtODkwYS1mMWEyYWVmNjBhMjQifQ.AKp-0zvKK4Hwa6qCgxskckD04Snf0gpSG7U1LOpcC_X")
            .encode();
        let mut req = Request::init();
        let mut req = unsafe {Pin::new_unchecked(&mut req)};
        req.as_mut().read(&mut &req_bytes[..]).await;

        assert_eq!(
            my_jwt.verified::<::serde_json::Value>(&req.as_ref()).unwrap_err().status,
            Status::Unauthorized
        );
    }

    #[test] async fn test_jwt_verify_senario() {
        use crate::prelude::*;
        use crate::{testing::*, Memory};
        use crate::typed::{ResponseBody, status::OK};

        use std::{sync::OnceLock, collections::HashMap, borrow::Cow};
        use crate::__rt__::Mutex;


        fn my_jwt() -> JWT {
            JWT::default("myverysecretjwtsecretkey")
        }

        #[derive(serde::Serialize, serde::Deserialize)]
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


        enum APIError {
            UserNotFound,
        }
        impl IntoResponse for APIError {
            fn into_response(self) -> Response {
                match self {
                    Self::UserNotFound     => Response::with(Status::InternalServerError).text("User was not found"),
                }
            }
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
                    first_name:   self.first_name.to_string(),
                    familly_name: self.familly_name.to_string(),
                }
            }
        }


        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct Profile {
            id:           usize,
            first_name:   String,
            familly_name: String,
        }
        impl ResponseBody for Profile {
            fn into_response_with(self, status: Status) -> Response {
                Response::with(status).json(self)
            }
        }

        async fn get_profile(jwt_payload: Memory<'_, MyJWTPayload>) -> Result<OK<Profile>, APIError> {
            let r = &mut *repository().await.lock().await;

            let user = r.get(&jwt_payload.user_id)
                .ok_or_else(|| APIError::UserNotFound)?;

            Ok(OK(user.profile()))
        }

        #[derive(serde::Deserialize, serde::Serialize/* for test */)]
        struct SigninRequest<'s> {
            first_name:   &'s str,
            familly_name: &'s str,
        } impl<'req> crate::FromRequest<'req> for SigninRequest<'req> {
            type Error = crate::FromRequestError;
            fn from_request(req: &'req Request) -> Result<Self, Self::Error> {
                serde_json::from_slice(
                    req.payload().ok_or_else(|| crate::FromRequestError::Static("No payload found"))?
                ).map_err(|e| crate::FromRequestError::Owned(e.to_string()))
            }
        }

        async fn signin(body: SigninRequest<'_>) -> utils::Text {
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

            utils::Text(issue_jwt_for_user(&user))
        }


        struct MyJWTFang(JWT);
        impl IntoFang for MyJWTFang {
            fn into_fang(self) -> Fang {
                Fang::front(move |req: &mut Request| {
                    let jwt_payload =  self.0.verified::<MyJWTPayload>(req)?;
                    req.memorize(jwt_payload);
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
            first_name:   String::from("ohkami"),
            familly_name: String::from("framework"),
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
            first_name:   String::from("Leonhard"),
            familly_name: String::from("Euler"),
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
            first_name:   String::from("ohkami"),
            familly_name: String::from("framework"),
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
                    first_name:   String::from("ohkami"),
                    familly_name: String::from("framework"),
                }),
                (2, User {
                    id:           2,
                    first_name:   String::from("Leonhard"),
                    familly_name: String::from("Euler"),
                }),
            ])
        }
    }
}
