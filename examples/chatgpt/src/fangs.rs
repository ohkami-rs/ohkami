use std::env;
use std::sync::OnceLock;
use ohkami::prelude::*;

#[derive(Clone)]
pub struct APIKey(pub &'static str);

impl APIKey {
    pub fn from_env() -> Self {
        static API_KEY: OnceLock<Option<String>> = OnceLock::new();

        let api_key = API_KEY.get_or_init(|| {
            match env::args().nth(1).as_deref() {
                Some("--api-key") => env::args().nth(2),
                _ => env::var("OPENAI_API_KEY").ok()
            }
        }).as_deref().expect("\
            OpenAI API key is not found\n\
            \n\
            [USAGE]\n\
            Run `cargo run` with one of \n\
              a. Set an environment variable `OPENAI_API_KEY` to your API key\n\
              b. Pass your API key by command line arguments `-- --api-key ＜here＞`\n\
        ");

        Self(api_key)
    }
}

impl FangAction for APIKey {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        req.context.set(self.clone());
        Ok(())
    }
}
