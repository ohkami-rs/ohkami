use std::default;

#[derive(Debug, Clone)]
pub struct CORS {
    pub allow_origins: &'static [&'static str],
    pub _private: ()
}

impl default::Default for CORS {
    fn default() -> Self {
        Self {
            allow_origins: &[],
            _private: ()
        }
    }
}