#![allow(non_snake_case)]

use std::sync::{OnceLock};


pub(crate) fn getGlobalFangs() -> &'static GlobalFangsImpl {
    GLOBAL_FANGS.get_or_init(|| GlobalFangs::default().into())
}

pub(crate) static GLOBAL_FANGS: OnceLock<GlobalFangsImpl> = OnceLock::new();
pub(crate) struct GlobalFangsImpl {
    pub(crate) CORS: &'static str,
} impl From<GlobalFangs> for GlobalFangsImpl {
    fn from(value: GlobalFangs) -> Self {
        Self {
            CORS: (value.cors)(CORS::new()).into_static(),
        }
    }
}

/// ```ignore
/// {
///     pub cors: fn(CORS) -> CORS
/// }
/// ```
pub struct GlobalFangs {
    pub cors: fn(CORS) -> CORS,
    // errors: , //for example rendering custom HTML for `NotFound` response
}
impl Default for GlobalFangs {
    fn default() -> Self {
        Self {
            cors: |new| new,
        }
    }
}
impl GlobalFangs {
    pub fn apply(self) {
        GLOBAL_FANGS.set(self.into())
            .ok().expect("Failed to apply GlobalFangs")
    }
}


pub struct CORS {
    AllowOrigin:      Option<&'static str>,
    AllowCredentials: Option<&'static str>,
    AllowHeaders:     Option<&'static str>,
    AllowMethods:     Option<&'static str>,
    ExposeHeaders:    Option<&'static str>,
    MaxAge:           Option<&'static str>,
} impl CORS {
    fn new() -> Self {
        Self {
            AllowOrigin:      None,
            AllowCredentials: None,
            AllowHeaders:     None,
            AllowMethods:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }
    fn into_static(self) -> &'static str {
        let headers = {
            let mut h = String::new();

            if let Some(value) = self.AllowOrigin {
                h.push_str("Access-Control-Allow-Origin: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }
            if let Some(value) = self.AllowCredentials {
                h.push_str("Access-Control-Allow-Credentials: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }
            if let Some(value) = self.AllowHeaders {
                h.push_str("Access-Control-Allow-Headers: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }
            if let Some(value) = self.AllowMethods {
                h.push_str("Access-Control-Allow-Methods: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }
            if let Some(value) = self.ExposeHeaders {
                h.push_str("Access-Control-Expose-Headers: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }
            if let Some(value) = self.MaxAge {
                h.push_str("Access-Control-Max-Age: ");
                h.push_str(value);
                h.push('\r'); h.push('\n');
            }

            h
        };
        Box::leak(Box::new(headers))
    }

    pub fn AllowOrigin(mut self, origin: &'static str) -> Self {
        self.AllowOrigin.replace(origin);
        self
    }
    pub fn AllowCredentials(mut self, credentials: &'static str) -> Self {
        self.AllowOrigin.replace(credentials);
        self
    }
    pub fn AllowHeaders(mut self, headers: &'static str) -> Self {
        self.AllowOrigin.replace(headers);
        self
    }
    pub fn AllowMethods(mut self, methods: &'static str) -> Self {
        self.AllowOrigin.replace(methods);
        self
    }
    pub fn ExposeHeaders(mut self, headers: &'static str) -> Self {
        self.AllowOrigin.replace(headers);
        self
    }
    pub fn MaxAge(mut self, delta_seconds: &'static str) -> Self {
        self.AllowOrigin.replace(delta_seconds);
        self
    }
}
