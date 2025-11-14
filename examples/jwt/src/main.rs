use ohkami::prelude::*;
use ohkami::fang::{Jwt, JwtToken};

fn jwt() -> Jwt<JwtPayload> {
    Jwt::default(std::env::var("JWT_SECRET").unwrap())
}

#[derive(Serialize, Deserialize)]
struct JwtPayload {
    sub: String,
    exp: u64,
}

trait JwtSub: 'static {
    fn sub() -> String;
}

struct DefaultJwtSub;
impl JwtSub for DefaultJwtSub {
    fn sub() -> String {"ohkami".to_string()}
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize))]
struct AuthResponse {
    token: JwtToken,
}

async fn auth<S: JwtSub>() -> Json<AuthResponse> {
    let token = jwt().issue(JwtPayload {
        sub: S::sub(),
        exp: ohkami::util::unix_timestamp() + 86400,
    });

    Json(AuthResponse { token })
}

async fn private(
    Context(_): Context<'_, JwtPayload>,
) -> &'static str {
    "Hello, private!"
}

fn ohkami<S: JwtSub>() -> Ohkami {
    Ohkami::new((
        "/auth".GET(auth::<S>),
        "/private".GET((jwt(), private)),
    ))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    ohkami::<DefaultJwtSub>().run("0.0.0.0:3000").await
}

#[cfg(test)]
mod test {
    use super::*;
    use ohkami::testing::*;

    /// regression test for https://github.com/ohkami-rs/ohkami/issues/433
    #[tokio::test]
    async fn test_large_jwt() {
        struct LargeJwtSub;
        impl JwtSub for LargeJwtSub {
            fn sub() -> String {
                const SENTENCE: &'static str = "\
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                    Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. \
                    Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris \
                    nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in \
                    reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla \
                    pariatur. Excepteur sint occaecat cupidatat non proident, sunt in \
                    culpa qui officia deserunt mollit anim id est laborum.\
                ";
                
                let sub = SENTENCE.repeat((1 << 11) / SENTENCE.len() + 1);
                
                // `sub` itself is already larger than default `request_bufsize`
                assert!(sub.len() > (1 << 11));
        
                sub
            }
        }

        dotenvy::dotenv().ok();
        let t = ohkami::<LargeJwtSub>().test();
        {            
            let req = TestRequest::GET("/auth");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
        }
        let token = {
            let req = TestRequest::GET("/auth");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            let AuthResponse { token } = res.json()
                .expect("`/auth` response doesn't contain a token");  
            token
        };
        {
            let req = TestRequest::GET("/private").header(
                "Authorization",
                format!("Bearer {token}") // <-- very large header field
            );
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::RequestHeaderFieldsTooLarge);
        }
        {
            let req = TestRequest::GET("/private").header(
                "Authorization",
                format!("Bearer {token}") // <-- very large header field
            );
            let res = t.oneshot_with(
                ohkami::Config {
                    request_bufsize: 1 << 14, // <-- set larger `request_bufsize` (16 KiB for example)
                    ..ohkami::Config::default()
                },
                req
            ).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), Some("Hello, private!"));
        };
    }
}
