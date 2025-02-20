#![allow(non_snake_case, non_camel_case_types)]

use crate::{Fang, FangProc, IntoResponse, Request, Response};
use crate::fang::{SendSyncOnNative, SendOnNative};
use std::{borrow::Cow, marker::PhantomData};
use serde::{Serialize, Deserialize};


/// # Builtin fang and helper for JWT config
/// 
/// <br>
/// 
/// ## fang
/// 
/// For each request, get JWT token and verify based on given config and `Payload: for<'de> Deserialize<'de>`.
/// 
/// ## helper
/// 
/// `.issue(/* Payload: Serialize */)` generates a JWT token on the config.
/// 
/// <br>
/// 
/// ## default config
/// 
/// - get token: from `Authorization: Bearer ＜here＞`
///   - customizable by `.get_token_by( 〜 )`
/// - verifying algorithm: `HMAC-SHA256`
///   - `HMAC-SHA{256, 384, 512}` are available now
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::typed::status;
/// use ohkami::fang::{JWT, JWTToken};
/// 
/// 
/// #[derive(Serialize, Deserialize)]
/// struct OurJWTPayload {
///     iat:       u64,
///     user_name: String,
/// }
/// 
/// fn our_jwt() -> JWT<OurJWTPayload> {
///     JWT::default("OUR_JWT_SECRET_KEY")
/// }
/// 
/// async fn hello(name: &str,
///     Context(auth): Context<'_, OurJWTPayload>
/// ) -> String {
///     format!("Hello {name}, you're authorized!")
/// }
/// 
/// # #[derive(Deserialize)]
/// # struct AuthRequest<'req> {
/// #     name: &'req str
/// # }
/// # 
/// # #[derive(Serialize)]
/// # struct AuthResponse {
/// #     token: JWTToken
/// # }
/// async fn auth(
///     JSON(req): JSON<AuthRequest<'_>>
/// ) -> Result<JSON<AuthResponse>, Response> {
///     Ok(JSON(AuthResponse {
///         token: our_jwt().issue(OurJWTPayload {
///             iat: ohkami::util::unix_timestamp(),
///             user_name: req.name.to_string()
///         })
///     }))
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/auth".GET(auth),
///         "/private".By(Ohkami::new((our_jwt(),
///             "/hello/:name".GET(hello),
///         )))
///     )).howl("localhost:3000").await
/// }
/// ```
pub struct JWT<Payload> {
    _payload:  PhantomData<Payload>,
    secret:    Cow<'static, str>,
    alg:       VerifyingAlgorithm,
    get_token: fn(&Request)->Option<&str>,

    #[cfg(feature="openapi")]
    openapi_security: crate::openapi::security::SecurityScheme
}
#[derive(Clone)]
enum VerifyingAlgorithm {
    HS256,
    HS384,
    HS512,
}

const _: () = {
    impl<Payload> Clone for JWT<Payload> {
        fn clone(&self) -> Self {
            Self {
                _payload:  PhantomData,
                secret:    self.secret.clone(),
                alg:       self.alg.clone(),
                get_token: self.get_token.clone(),

                #[cfg(feature="openapi")]
                openapi_security: self.openapi_security.clone()
            }
        }
    }

    impl<
        Inner: FangProc + SendOnNative,
        Payload: Serialize + for<'de> Deserialize<'de> + SendSyncOnNative + 'static,
    > Fang<Inner> for JWT<Payload> {
        type Proc = JWTProc<Inner, Payload>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            JWTProc { inner, jwt: self.clone() }
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            operation.security(self.openapi_security(), &[])
        }
    }

    pub struct JWTProc<
        Inner: FangProc,
        Payload: Serialize + for<'de> Deserialize<'de>,
    > {
        inner: Inner,
        jwt:   JWT<Payload>,
    }
    impl<
        Inner: FangProc + SendOnNative,
        Payload: Serialize + for<'de> Deserialize<'de> + SendSyncOnNative + 'static,
    > FangProc for JWTProc<Inner, Payload> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            let jwt_payload = match self.jwt.verified(req) {
                Ok(payload) => payload,
                Err(errres) => return errres
            };
            req.context.set(jwt_payload);

            self.inner.bite(req).await.into_response()
        }
    }
};

impl<Payload> JWT<Payload> {
    /// Just `new_256`; use HMAC-SHA256 as verifying algorithm
    #[inline] pub fn default(secret: impl Into<Cow<'static, str>>) -> Self {
        Self::new_256(secret)
    }
    /// Use HMAC-SHA256 as verifying algorithm
    pub fn new_256(secret: impl Into<Cow<'static, str>>) -> Self {
        Self::new(VerifyingAlgorithm::HS256, secret)
    }
    /// Use HMAC-SHA384 as verifying algorithm
    pub fn new_384(secret: impl Into<Cow<'static, str>>) -> Self {
        Self::new(VerifyingAlgorithm::HS384, secret)
    }
    /// Use HMAC-SHA512 as verifying algorithm
    pub fn new_512(secret: impl Into<Cow<'static, str>>) -> Self {
        Self::new(VerifyingAlgorithm::HS512, secret)
    }

    /// Customize get-token process in JWT verifying.
    /// 
    /// *default*: `req.headers.Authorization()?.strip_prefix("Bearer ")`
    pub fn get_token_by(
        mut self,
        get_token: fn(&Request)->Option<&str>,

        #[cfg(feature="openapi")]
        openapi_security: crate::openapi::security::SecurityScheme
    ) -> Self {
        #[cfg(feature="openapi")] {
            self.openapi_security = openapi_security;
        }
        self.get_token = get_token;
        self
    }

    pub fn get_token_fn(&self) -> &fn(&Request)->Option<&str> {
        &self.get_token
    }

    #[cfg(feature="openapi")]
    pub fn openapi_security(&self) -> crate::openapi::SecurityScheme {
        self.openapi_security.clone()
    }

    fn new(alg: VerifyingAlgorithm, secret: impl Into<Cow<'static, str>>) -> Self {
        #[inline(always)]
        fn get_token(req: &Request) -> Option<&str> {
            req.headers.Authorization()?.strip_prefix("Bearer ")
        }

        Self {
            alg,
            get_token,

            _payload: PhantomData,
            secret: secret.into(),

            #[cfg(feature="openapi")]
            openapi_security: crate::openapi::security::SecurityScheme::Bearer(
                "jwtAuth",
                Some("JWT")
            )
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
            VerifyingAlgorithm::HS256 => r#"{"typ":"JWT","alg":"HS256"}"#,
            VerifyingAlgorithm::HS384 => r#"{"typ":"JWT","alg":"HS384"}"#,
            VerifyingAlgorithm::HS512 => r#"{"typ":"JWT","alg":"HS512"}"#,
        }
    }
}

/// Struct holding JWT token issued by `JWT::issue`.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JWTToken(String);
const _: () = {
    impl std::fmt::Display for JWTToken {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.0, f)
        }
    }

    impl std::ops::Deref for JWTToken {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Into<String> for JWTToken {
        fn into(self) -> String {
            self.0
        }
    }
};

impl<Payload: Serialize> JWT<Payload> {
    /// Build JWT token with the payload.
    #[inline] pub fn issue(self, payload: Payload) -> JWTToken {
        let unsigned_token = {
            let mut ut = crate::util::base64_url_encode(self.header_str());
            ut.push('.');
            ut.push_str(&crate::util::base64_url_encode(::serde_json::to_vec(&payload).expect("Failed to serialze payload")));
            ut
        };

        let signature = {
            use ::sha2::{Sha256, Sha384, Sha512};
            use ::hmac::{Hmac, Mac};

            match &self.alg {
                VerifyingAlgorithm::HS256 => crate::util::base64_url_encode({
                    let mut s = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
                    s.update(unsigned_token.as_bytes());
                    s.finalize().into_bytes()
                }),
                VerifyingAlgorithm::HS384 => crate::util::base64_url_encode({
                    let mut s = Hmac::<Sha384>::new_from_slice(self.secret.as_bytes()).unwrap();
                    s.update(unsigned_token.as_bytes());
                    s.finalize().into_bytes()
                }),
                VerifyingAlgorithm::HS512 => crate::util::base64_url_encode({
                    let mut s = Hmac::<Sha512>::new_from_slice(self.secret.as_bytes()).unwrap();
                    s.update(unsigned_token.as_bytes());
                    s.finalize().into_bytes()
                }),
            }
        };
        
        let mut token = unsigned_token;
        token.push('.');
        token.push_str(&signature);
        JWTToken(token)
    }
}

impl<Payload: for<'de> Deserialize<'de>> JWT<Payload> {
    /// Verify JWT in requests' `Authorization` header and early return error response if
    /// it's missing or malformed.
    pub fn verify(&self, req: &Request) -> Result<(), Response> {
        let _ = self.verified(req)?;
        Ok(())
    }

    /// Verify JWT in requests' `Authorization` header and early return error response if
    /// it's missing or malformed.
    /// 
    /// Then it's valid, this returns decoded paylaod of the JWT as `Payload`.
    pub fn verified(&self, req: &Request) -> Result<Payload, Response> {
        (! req.method.isOPTIONS()).then_some(()).ok_or_else(Response::OK)?;

        const UNAUTHORIZED_MESSAGE: &str = "missing or malformed jwt";

        let mut parts = (self.get_token)(req)
            .ok_or_else(|| Response::Unauthorized().with_text(UNAUTHORIZED_MESSAGE))?
            .split('.');

        type Header  = ::serde_json::Value;
        type Payload = ::serde_json::Value;
        fn part_value(part: &str) -> Result<::serde_json::Value, Response> {
            let part = crate::util::base64_url_decode(part)
                .map_err(|_| Response::BadRequest().with_text("invalid base64"))?;
            ::serde_json::from_slice(&part)
                .map_err(|_| Response::BadRequest().with_text("invalid json"))
        }

        let header_part = parts.next()
            .ok_or_else(Response::Unauthorized)?;
        let header: Header = part_value(header_part)?;
        if header.get("typ").is_some_and(|typ| !typ.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
            return Err(Response::BadRequest())
        }
        if header.get("cty").is_some_and(|cty| !cty.as_str().unwrap_or_default().eq_ignore_ascii_case("JWT")) {
            return Err(Response::BadRequest())
        }
        if header.get("alg").ok_or_else(Response::Unauthorized)? != self.alg_str() {
            return Err(Response::BadRequest())
        }

        let payload_part = parts.next()
            .ok_or_else(Response::Unauthorized)?;
        let payload: Payload = part_value(payload_part)?;
        let now = crate::util::unix_timestamp();
        if payload.get("nbf").is_some_and(|nbf| nbf.as_u64().unwrap_or(0) > now) {
            return Err(Response::Unauthorized().with_text(UNAUTHORIZED_MESSAGE))
        }
        if payload.get("exp").is_some_and(|exp| exp.as_u64().unwrap_or(u64::MAX) <= now) {
            return Err(Response::Unauthorized().with_text(UNAUTHORIZED_MESSAGE))
        }
        if payload.get("iat").is_some_and(|iat| iat.as_u64().unwrap_or(0) > now) {
            return Err(Response::Unauthorized().with_text(UNAUTHORIZED_MESSAGE))
        }

        let signature_part = parts.next()
            .ok_or_else(Response::Unauthorized)?;
        let requested_signature = crate::util::base64_url_decode(signature_part)
            .map_err(|_| Response::Unauthorized())?;

        let is_correct_signature = {
            use ::sha2::{Sha256, Sha384, Sha512};
            use ::hmac::{Hmac, Mac};

            match self.alg {
                VerifyingAlgorithm::HS256 => {
                    let mut hs = Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().as_slice() == &requested_signature
                }
                VerifyingAlgorithm::HS384 => {
                    let mut hs = Hmac::<Sha384>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().as_slice() == &requested_signature
                }
                VerifyingAlgorithm::HS512 => {
                    let mut hs = Hmac::<Sha512>::new_from_slice(self.secret.as_bytes()).unwrap();
                    hs.update(header_part.as_bytes());
                    hs.update(b".");
                    hs.update(payload_part.as_bytes());
                    hs.finalize().into_bytes().as_slice() == &requested_signature
                }
            }
        };
        
        if !is_correct_signature {
            return Err(Response::Unauthorized().with_text(UNAUTHORIZED_MESSAGE))
        }

        let payload = ::serde_json::from_value(payload).map_err(|_| Response::InternalServerError())?;
        Ok(payload)
    }
}




#[cfg(test)]
#[test] fn jwt_fang_bound() {
    use crate::fang::{Fang, BoxedFPC};
    fn assert_fang<T: Fang<BoxedFPC>>() {}

    assert_fang::<JWT<String>>();
}

#[cfg(debug_assertions)]
#[cfg(all(feature="__rt_native__", feature="DEBUG"))]
#[cfg(test)] mod test {
    use super::{JWT, JWTToken};

    #[test] fn test_jwt_issue() {
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
            &*JWT::default("secret").issue(::serde_json::json!({"name":"kanarus","id":42,"iat":1516239022})),
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE1MTYyMzkwMjIsImlkIjo0MiwibmFtZSI6ImthbmFydXMifQ.dt43rLwmy4_GA_84LMC1m5CwVc59P9as_nRFldVCH7g"
        }
    }

    #[test] fn test_jwt_verify() {
        use crate::{Request, testing::TestRequest, Status};
        use std::pin::Pin;

        let my_jwt = JWT::<::serde_json::Value>::default("ohkami-realworld-jwt-authorization-secret-key");

        let req_bytes = TestRequest::GET("/")
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MDY4MTEwNzUsInVzZXJfaWQiOiI5ZmMwMDViMi1mODU4LTQzMzYtODkwYS1mMWEyYWVmNjBhMjQifQ.AKp-0zvKK4Hwa6qCgxskckD04Snf0gpSG7U1LOpcC_I")
            .encode();
        let mut req = Request::init(crate::util::IP_0000);
        let mut req = unsafe {Pin::new_unchecked(&mut req)};
        crate::__rt__::testing::block_on({
            req.as_mut().read(&mut &req_bytes[..])
        });

        assert_eq!(
            my_jwt.verified(&req.as_ref()).unwrap(),
            ::serde_json::json!({ "iat": 1706811075, "user_id": "9fc005b2-f858-4336-890a-f1a2aef60a24" })
        );

        let req_bytes = TestRequest::GET("/")
            // Modifed last `I` of the value above to `X`
            .header("Authorization", "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE3MDY4MTEwNzUsInVzZXJfaWQiOiI5ZmMwMDViMi1mODU4LTQzMzYtODkwYS1mMWEyYWVmNjBhMjQifQ.AKp-0zvKK4Hwa6qCgxskckD04Snf0gpSG7U1LOpcC_X")
            .encode();
        let mut req = Request::init(crate::util::IP_0000);
        let mut req = unsafe {Pin::new_unchecked(&mut req)};
        crate::__rt__::testing::block_on({
            req.as_mut().read(&mut &req_bytes[..])
        });

        assert_eq!(
            my_jwt.verified(&req.as_ref()).unwrap_err().status,
            Status::Unauthorized
        );
    }

    #[test] fn test_jwt_verify_senario() {
        use crate::prelude::*;
        use crate::testing::*;
        use std::{sync::OnceLock, sync::Mutex, collections::HashMap, borrow::Cow};

        #[cfg(feature="openapi")]
        use crate::openapi;


        fn my_jwt() -> JWT<MyJWTPayload> {
            JWT::default("myverysecretjwtsecretkey")
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        struct MyJWTPayload {
            iat:     u64,
            user_id: usize,
        }

        fn issue_jwt_for_user(user: &User) -> JWTToken {
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
                    Self::UserNotFound => Response::InternalServerError().with_text("User was not found"),
                }
            }

            #[cfg(feature="openapi")]
            fn openapi_responses() -> crate::openapi::Responses {
                crate::openapi::Responses::new([
                    (500, crate::openapi::Response::when("User was not found")
                        .content("text/plain", crate::openapi::string()))
                ])
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
        }
        impl User {
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
        #[cfg(feature="openapi")]
        impl openapi::Schema for Profile {
            fn schema() -> impl Into<openapi::schema::SchemaRef> {
                openapi::component("Profile", openapi::object()
                    .property("id", openapi::integer().minimum(0))
                    .property("first_name", openapi::string())
                    .property("familly_name", openapi::string())
                )
            }
        }

        async fn get_profile(
            Context(jwt_payload): Context<'_, MyJWTPayload>
        ) -> Result<JSON<Profile>, APIError> {
            let r = &mut *repository().await.lock().unwrap();

            let user = r.get(&jwt_payload.user_id)
                .ok_or_else(|| APIError::UserNotFound)?;

            Ok(JSON(user.profile()))
        }

        #[derive(serde::Deserialize, serde::Serialize/* for test */)]
        struct SigninRequest<'s> {
            first_name:   &'s str,
            familly_name: &'s str,
        }
        #[cfg(feature="openapi")]
        impl<'s> openapi::Schema for SigninRequest<'s> {
            fn schema() -> impl Into<openapi::schema::SchemaRef> {
                openapi::component("SigninRequest", openapi::object()
                    .property("first_name", openapi::string())
                    .property("familly_name", openapi::string())
                )
            }
        }

        async fn signin(
            JSON(req): JSON<SigninRequest<'_>>
        ) -> String/* for test */ {
            let r = &mut *repository().await.lock().unwrap();

            let user: Cow<'_, User> = match r.iter().find(|(_, u)|
                u.first_name   == req.first_name &&
                u.familly_name == req.familly_name
            ) {
                Some((_, u)) => Cow::Borrowed(u),
                None => {
                    let new_user_id = match r.keys().max() {
                        Some(max) => max + 1,
                        None      => 1,
                    };

                    let new_user = User {
                        id:           new_user_id,
                        first_name:   req.first_name.to_string(),
                        familly_name: req.familly_name.to_string(), 
                    };

                    r.insert(new_user_id, new_user.clone());

                    Cow::Owned(new_user)
                }
            };

            issue_jwt_for_user(&user).into()
        }

        let t = Ohkami::new((
            "/signin".By(Ohkami::new(
                "/".PUT(signin),
            )),
            "/profile".By(Ohkami::new((my_jwt(),
                "/".GET(get_profile),
            ))),
        )).test();
        

        crate::__rt__::testing::block_on(async {

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
                &*repository().await.lock().unwrap(),
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
                &*repository().await.lock().unwrap(),
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
                &*repository().await.lock().unwrap(),
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
        });
    }
}
