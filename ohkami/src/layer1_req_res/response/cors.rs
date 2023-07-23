#![allow(non_snake_case)]

use crate::Method;


pub struct CORS {
    pub(crate) AllowOrigin: &'static str,
    AllowCredentials:       bool,
    AllowHeaders:           Option<Vec<&'static str>>,
    AllowMethods:           Option<Vec<Method>>,
    MaxAge:                 Option<u32>,
} impl CORS {
    pub(crate) fn new(AllowOrigin: &'static str) -> Self {
        Self {
            AllowOrigin,
            AllowCredentials: false,
            AllowHeaders:     None,
            AllowMethods:     None,
            MaxAge:           None,
        }
    }
    pub(crate) fn into_static(self) -> &'static str {
        let headers = {
            let mut h = format!("Access-Control-Allow-Origin: {}\r\n", self.AllowOrigin);
            if self.AllowCredentials {
                h.push_str("Access-Control-Allow-Credentials: true\r\n");
            }
            if let Some(seconds) = self.MaxAge {
                h.push_str(&format!("Access-Control-Max-Age: {seconds}\r\n"));
            }
            if let Some(headers) = self.AllowHeaders {
                let headers = headers.join(",");
                h.push_str(&format!("Access-Control-Allow-Headers: {headers}\r\n"));
            }
            if let Some(methods) = self.AllowMethods {
                let methods = methods.into_iter().map(|m| m.to_string()).collect::<Vec<_>>().join(",");
                h.push_str(&format!("Access-Control-Allow-Methods: {methods}\r\n"));
            }
            h
        };
        Box::leak(Box::new(headers))
    }
}

impl CORS {
    pub fn AllowCredentials(mut self) -> Self {
        self.AllowCredentials = true;
        self
    }
    pub fn AllowHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.AllowHeaders.replace(headers.to_vec());
        self
    }
    pub fn AllowMethods<const N: usize>(mut self, methods: [crate::Method; N]) -> Self {
        self.AllowMethods.replace(methods.to_vec());
        self
    }
    pub fn MaxAge(mut self, delta_seconds: u32) -> Self {
        self.MaxAge.replace(delta_seconds);
        self
    }
}
