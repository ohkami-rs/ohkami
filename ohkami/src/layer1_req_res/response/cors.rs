#![allow(non_snake_case)]


pub struct CORS {
    AllowOrigin:      Option<&'static str>,
    AllowCredentials: Option<&'static str>,
    AllowHeaders:     Option<&'static str>,
    AllowMethods:     Option<&'static str>,
    ExposeHeaders:    Option<&'static str>,
    MaxAge:           Option<&'static str>,
} impl CORS {
    pub(crate) fn new() -> Self {
        Self {
            AllowOrigin:      None,
            AllowCredentials: None,
            AllowHeaders:     None,
            AllowMethods:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }
    pub(crate) fn into_static(self) -> &'static str {
        let headers = {
            let mut h = String::new();
            if let Some(value) = self.AllowOrigin {
                h.push_str(&format!("Access-Control-Allow-Origin: {value}\r\n"));
            }
            if let Some(value) = self.AllowCredentials {
                h.push_str(&format!("Access-Control-Allow-Credentials: {value}\r\n"));
            }
            if let Some(value) = self.AllowHeaders {
                h.push_str(&format!("Access-Control-Allow-Headers: {value}\r\n"));
            }
            if let Some(value) = self.AllowMethods {
                h.push_str(&format!("Access-Control-Allow-Methods: {value}\r\n"));
            }
            if let Some(value) = self.ExposeHeaders {
                h.push_str(&format!("Access-Control-Expose-Headers: {value}\r\n"));
            }
            if let Some(value) = self.MaxAge {
                h.push_str(&format!("Access-Control-Max-Age: {value}\r\n"));
            }
            h
        };
        Box::leak(Box::new(headers))
    }
}

impl CORS {
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
