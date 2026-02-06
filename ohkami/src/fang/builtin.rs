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
    let Some(("http" | "https", rest)) = origin.split_once("://") else {
        return Err(
            "invalid origin: 'http' or 'https' scheme is required"
        )
    };
    let (host, port) = rest
        .split_once(':')
        .map_or((rest, None), |(h, p)| (h, Some(p)));
    if port.is_some_and(|p| !p.chars().all(|c| c.is_ascii_digit())) {
        return Err("invalid origin: port must be a number");
    }
    if !host.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(
            "invalid origin: host must start with an alphabetic character"
        );
    }
    if !host.split('.').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
    }) {
        if host.contains(['/', '?', '#']) {
            // helpful error message for common mistake
            return Err(
                "invalid origin: path, query and fragment are not allowed"
            );
        } else {
            return Err("invalid origin: invalid host");
        }
    }
    Ok(())
}
