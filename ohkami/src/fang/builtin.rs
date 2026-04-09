mod basicauth;
pub use basicauth::BasicAuth;

mod cors;
pub use cors::Cors;

mod csrf;
pub use csrf::Csrf;

mod jwt;
pub use jwt::{Jwt, JwtToken};

mod context;
pub use context::Context;

pub mod enamel;
pub use enamel::Enamel;

#[cfg(feature = "__rt_native__")]
mod timeout;
#[cfg(feature = "__rt_native__")]
pub use timeout::Timeout;

fn validate_origin(origin: &str) -> Result<(), &'static str> {
    //Adds a check for the first characters being http or https, so it cannot be malformed like foobarhttp://example.org/
    if !origin.starts_with("http") {
        return Err("invalid origin: 'http' or 'https' scheme is required at the start of the string.")
    }
    let Some(("http" | "https", rest)) = origin.split_once("://") else {
        return Err("invalid origin: 'http' or 'https' scheme is required.");
    };
    //Adds a check for the maximum length of a domain
    if rest.chars().count() > 253 {
        return Err("invalid origin: maximum length 253 for domain exceeded.")
    }
    let (host, port) = rest
        .split_once(':')
        .map_or((rest, None), |(h, p)| (h, Some(p)));
     if port.is_some_and(|p| p != "*" && p.parse::<u16>().is_err()) {
        return Err("invalid origin: port must be a number between 0 and 65535 or wildcard '*'.");
    }
    if !host.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '*') {
        return Err("invalid origin: host must start with an alphanumeric character or wildcard '*'.");
    }
    if !host.split('.').all(|part| {
        !part.is_empty()
        && part.chars().all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '*')
        && part.chars().count() <= 63)
    }) {
        return if host.contains(['/', '?', '#']) {
            // helpful error message for common mistake
            Err("invalid origin: path, query and fragment are not allowed.")
        } else {
            Err("invalid origin: invalid host.")
        }
    }

    if host.contains('.') {
        let Some((subdomain, sld)) = host.split_once('.') else {
            return Err("invalid origin: invalid host")
        };

        if sld.split('.').all(|part| part.chars().all(|c| c.is_numeric())) {
            if subdomain == "*" {
                return Err("invalid origin: subdomain wildcard not allowed in IP")
            }
        }
    }

    Ok(())
}
