use ohkami::prelude::*;
use ohkami::fang::{JWT, JWTToken};

fn jwt() -> JWT<JwtPayload> {
    JWT::default(std::env::var("JWT_SECRET").unwrap())
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
    token: JWTToken,
}

async fn auth<S: JwtSub>() -> JSON<AuthResponse> {
    let token = jwt().issue(JwtPayload {
        sub: S::sub(),
        exp: ohkami::util::unix_timestamp() + 86400,
    });

    JSON(AuthResponse { token })
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
    ohkami::<DefaultJwtSub>().howl("0.0.0.0:3000").await
}

#[cfg(test)]
mod test {
    use super::*;
    use ohkami::testing::*;

    /// regression test for https://github.com/ohkami-rs/ohkami/issues/433
    /// 
    /// run with `OHKAMI_REQUEST_BUFSIZE=4096` or larger
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

        let req = TestRequest::GET("/auth");
        let res = t.oneshot(req).await;
        let AuthResponse { token } = res.json()
            .expect("`/auth` response doesn't contain a token");

        let req = TestRequest::GET("/private")
            .header("Authorization", format!("Bearer {token}"));
        let res = t.oneshot(req).await;
        assert_eq!(res.status().code(), 200);
        assert_eq!(res.text(), Some("Hello, private!"));
    }
}
