use std::env;
use std::sync::OnceLock;
use ohkami::prelude::*;


#[derive(Clone)]
pub struct WithAPIKey {
    api_key: &'static str,
}
impl WithAPIKey {
    pub fn from_env() -> Option<Self> {
        static API_KEY: OnceLock<Option<String>> = OnceLock::new();

        Some(Self {
            api_key: API_KEY.get_or_init(|| {
                match env::args().nth(1).as_deref() {
                    Some("--api-key") => env::args().nth(2),
                    _ => env::var("OPENAI_API_KEY").ok()
                }
            }).as_deref()?
        })
    }
}
impl FangAction for WithAPIKey {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        req.memorize(self.api_key);
        Ok(())
    }
}
