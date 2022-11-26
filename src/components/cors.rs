use std::default;

#[derive(Clone)]
pub struct CORS<'cors> {
    pub allow_origins: &'cors [&'static str],
    pub _private: ()
}

impl<'cors> default::Default for CORS<'cors> {
    fn default() -> Self {
        Self {
            allow_origins: &[],
            _private: ()
        }
    }
}