#![allow(non_snake_case)]

use std::sync::{OnceLock};

use crate::layer1_req_res::ErrResponse;


pub(crate) fn getGlobalFangs() -> &'static GlobalFangsImpl {
    GLOBAL_FANGS.get_or_init(|| GlobalFangs::new().into())
}

pub(crate) static GLOBAL_FANGS: OnceLock<GlobalFangsImpl> = OnceLock::new();
pub(crate) struct GlobalFangsImpl {
    pub(crate) CORS: &'static str,
    pub(crate) NotFound: fn(ErrResponse) -> ErrResponse,
} impl From<GlobalFangs> for GlobalFangsImpl {
    fn from(value: GlobalFangs) -> Self {
        Self {
            CORS: (value.cors)(CORS::new()).into_static(),
            NotFound: value.custom_notfound,
        }
    }
}

/// <br/>
/// 
/// ```ignore
/// use ohkami::{GlobalFangs, Route, Ohkami};
/// use crate::handlers::{
///     health_handler::{
///         health_check,
///     },
///     user_handler::{
///         create_user, get_user, update_user, delete_user,
///     },
/// };
/// 
/// #[tokio::main]
/// async fn main() {
///     GlobalFangs::new()
///         .CORS(|c| c
///             .AllowOrigin("https://myapp.example")
///             .MaxAge("1024"))
///         .NotFound(|nf| nf
///             .HTML(include_str!("../404.html")))
///         .apply();
/// 
///     Ohkami::new()(
///         "/hc"
///             .GET(health_check),
///         "/api/users"
///             .POST(create_user),
///         "/api/users/:id"
///             .GET(get_user)
///             .PATCH(update_user)
///             .DELETE(delete_user),
///     ).howl(8080).await
/// }
/// ```
pub struct GlobalFangs {
    cors: fn(CORS) -> CORS,
    custom_notfound: fn(ErrResponse) -> ErrResponse,
} impl GlobalFangs {
    pub fn new() -> Self {
        Self {
            cors: |f| f,
            custom_notfound: |f| f,
        }
    }

    /// Apply this `GlobalFang` to entire the application
    pub fn apply(self) {
        GLOBAL_FANGS.set(self.into())
            .ok().expect("Failed to apply GlobalFangs")
    }
} impl GlobalFangs {
    pub fn CORS(mut self, cors_config: fn(CORS) -> CORS) -> Self {
        self.cors = cors_config;
        self
    }
    pub fn NotFound(mut self, custom_not_found: fn(ErrResponse) -> ErrResponse) -> Self {
        self.custom_notfound = custom_not_found;
        self
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
