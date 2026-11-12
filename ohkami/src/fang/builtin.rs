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

/* #660 will replace this with an original struct
   (providing almost the same interface as this using original implementations internally)
*/
// Just wrapping `https::uri::Uri` for now to skip most difficult things in parsing,
// so we only have to handle HTTP-origin-specific rules on the top of the generic `Uri`.
#[derive(Clone, Debug)]
pub struct Origin(http::uri::Uri);

#[derive(Debug)]
pub enum OriginError {
    InvalidUri(http::uri::InvalidUri),
    FaultyScheme,
    FaultyUriLength,
    FaultyPort,
    FaultyIp,
    InvalidHost,
    InvalidSuffix,
}

#[derive(PartialEq)]
pub enum Scheme {
    Http,
    Https,
}

impl std::fmt::Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Scheme::Http => "http",
                Scheme::Https => "https",
            }
        )
    }
}

impl Origin {
    const MIN_IP_PART_COUNT: usize = 4;
    const MAX_HOST_LENGTH: usize = 253;
    const MAX_HOST_LABEL_LENGTH: usize = 63;

    /// Parse string into HTTP origin.
    ///
    /// # Examples
    /// ```ignore
    /// fn run() {
    ///     Origin::new("https://localhost:3000").unwrap();
    /// }
    /// ```
    /// # Errors
    ///
    /// This function will return an error if the given URI string fails the validation included in this function.
    /// Rules include:
    ///
    /// - Generalistic http::uri::Uri rules for URI's.
    /// - URI must not contain path, query or fragment, and instead must be e.g. "https://sub.example.com"
    /// - Scheme must be either HTTP or HTTPS.
    /// - Host length mustn't exceed 253 characters in total.
    /// - Host Label parts mustn't exceed 63 characters per.
    /// - Ports must be numeric and <= 65535 (`u16::MAX`).
    /// - IP strings like 192.168.1.0 must adhere to their respective rules for IPv4 or IPv6.
    /// - Labels consist of only letters, digits, or hyphens. Cannot start or end with hyphens.
    ///
    fn new(s: &str) -> Result<Self, OriginError> {
        use http::uri::{Scheme, Uri};

        let uri = s.parse::<Uri>().map_err(OriginError::InvalidUri)?;

        // Additional validation
        // Validate scheme is HTTP or HTTPS
        if uri
            .scheme()
            .is_none_or(|s| s != &Scheme::HTTP && s != &Scheme::HTTPS)
        {
            return Err(OriginError::FaultyScheme);
        }

        if s.split_once("://").unwrap().1.contains(['/', '?', '#']) {
            // If given Origin string contains a path, fragment or query
            return Err(OriginError::InvalidSuffix);
        }

        let Some(host) = uri.host() else {
            return Err(OriginError::InvalidHost);
        };

        // Validate max host length
        if host.chars().count() > Self::MAX_HOST_LENGTH {
            return Err(OriginError::FaultyUriLength);
        }

        let host_labels = host.strip_suffix('.').unwrap_or(host).split('.');

        if !host_labels.clone().all(|label| {
            !label.is_empty()
                && label
                    .chars()
                    .all(|c| matches!(c, |'0'..='9'| 'a'..='z' | '-'))
                && (label.len() <= Self::MAX_HOST_LABEL_LENGTH)
                && !label.starts_with('-')
                && !label.ends_with('-')
        }) {
            return Err(OriginError::InvalidHost);
        }

        if host_labels.clone().all(|label| label.parse::<u8>().is_ok())
            && host_labels.count() < Self::MIN_IP_PART_COUNT
        {
            return Err(OriginError::FaultyIp);
        }

        // WORKAROUND: At now, `http::uri::Uri` silently parses with an invalid port value,
        // and then its `.port()` or `.port_u16()` just returns `None`.
        // (https://github.com/hyperium/http/issues/509)
        if uri
            .authority()
            .is_some_and(|authority| authority.as_str().contains(':') && uri.port().is_none())
        {
            return Err(OriginError::FaultyPort);
        }

        Ok(Self(uri))
    }

    fn scheme(&self) -> Scheme {
        if self.0.scheme() == Some(&http::uri::Scheme::HTTP) {
            Scheme::Http
        } else {
            Scheme::Https // definitely Https because of `Self::new` parser logic
        }
    }

    fn port(&self) -> Option<u16> {
        self.0.port_u16()
    }

    fn host(&self) -> &str {
        // assured by `Origin::new`
        self.0.host().unwrap()
    }

    fn authority(&self) -> &http::uri::Authority {
        self.0.authority().unwrap()
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.scheme(), self.authority())
    }
}

// `http::uri::InvalidUri` doesn't implement `PartialEq`
// (https://github.com/hyperium/http/issues/849)
impl PartialEq for OriginError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidUri(a), Self::InvalidUri(b)) => a.to_string() == b.to_string(),
            (Self::FaultyScheme, Self::FaultyScheme)
            | (Self::FaultyUriLength, Self::FaultyUriLength)
            | (Self::FaultyPort, Self::FaultyPort)
            | (Self::FaultyIp, Self::FaultyIp)
            | (Self::InvalidHost, Self::InvalidHost)
            | (Self::InvalidSuffix, Self::InvalidSuffix) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for OriginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OriginError::InvalidUri(_) => {
                    "Invalid URI."
                }
                OriginError::FaultyScheme => {
                    "Please use HTTP or HTTPS as scheme."
                }
                OriginError::FaultyUriLength => {
                    "URI length mustn't exceed 253 characters in total."
                }
                OriginError::FaultyPort => {
                    "Port number was expected."
                }
                OriginError::FaultyIp => {
                    "Ip was misformatted."
                }
                OriginError::InvalidHost => {
                    "Invalid URI for usage in Origin. (e.g. Part length mustn't exceed 63 characters, and start and end with a digit or letter)"
                }
                OriginError::InvalidSuffix => {
                    "URI should not contain a path, query or segment. (e.g. 'https://example.com/hello' should be 'https://example.com' )"
                }
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Origin, OriginError};

    #[test]
    fn origin_validate_valid_origins_with_ip() {
        assert!(Origin::new("http://192.168.1.2").is_ok());
        assert!(Origin::new("https://192.168.1.2").is_ok());
        assert!(Origin::new("https://192.168.1.2:3000").is_ok());
        assert!(Origin::new("https://192.168.1.2:80").is_ok());
    }

    #[test]
    fn origin_validate_valid_origins_with_domain() {
        assert!(Origin::new("http://example.com").is_ok());
        assert!(Origin::new("https://example.com").is_ok());
        assert!(Origin::new("https://example.com:3000").is_ok());
        assert!(Origin::new("https://example.com:80").is_ok());
        assert!(Origin::new("https://sub.example.com").is_ok());
        assert!(Origin::new("https://sub.example.com:3000").is_ok());
        assert!(Origin::new("https://3xample.com").is_ok());
        assert!(Origin::new("https://3xample.com").is_ok());
        assert!(Origin::new("https://3xample.com:8080").is_ok());
        assert!(Origin::new("https://sub.3xample.com:8080").is_ok());
    }

    #[test]
    fn origin_validate_valid_origins_with_fqdn() {
        assert!(Origin::new("http://localhost").is_ok());
        assert!(Origin::new("https://localhost").is_ok());
        assert!(Origin::new("http://localhost:3000").is_ok());
        assert!(Origin::new("https://localhost:3000").is_ok());
        assert!(Origin::new("http://example").is_ok());
        assert!(Origin::new("https://example").is_ok());
        assert!(Origin::new("http://example:3000").is_ok());
        assert!(Origin::new("https://example:3000").is_ok());
        assert!(Origin::new("http://example.").is_ok());
        assert!(Origin::new("https://example.").is_ok());
        assert!(Origin::new("http://example.:3000").is_ok());
        assert!(Origin::new("https://example.:3000").is_ok());
    }

    #[test]
    fn origin_invalid_origin_ip_invalidation() {
        assert_eq!(
            Origin::new("https://192.*.1.15:8080").unwrap_err(),
            OriginError::InvalidHost,
        );
        assert_eq!(
            Origin::new("https://*.168.1.15:8080").unwrap_err(),
            OriginError::InvalidHost
        );
    }

    #[test]
    fn origin_host_invalidation() {
        assert_eq!(
            Origin::new("https://test.example.*:8080").unwrap_err(),
            OriginError::InvalidHost
        );

        assert_eq!(
            Origin::new("https://test.*.com:8080").unwrap_err(),
            OriginError::InvalidHost
        );

        assert!(Origin::new("https://ëxample.com:8080").is_err());

        assert!(
            Origin::new("http://%example.com").is_err() //Gives InvalidUri error, which's enums aren't public so unable to directly compare.
        );

        assert_eq!(
            Origin::new("https://a..example.com").unwrap_err(),
            OriginError::InvalidHost
        );

        assert_eq!(
            Origin::new("https://..example.com").unwrap_err(),
            OriginError::InvalidHost
        );
    }

    #[test]
    fn origin_scheme_invalidation() {
        assert_eq!(
            Origin::new("foobarhttp://example.com").unwrap_err(),
            OriginError::FaultyScheme
        );
        assert_eq!(
            Origin::new("example.com").unwrap_err(),
            OriginError::FaultyScheme
        );
        assert_eq!(
            Origin::new("sub.example.com").unwrap_err(),
            OriginError::FaultyScheme
        );
        assert_eq!(
            Origin::new("192.168.1.0").unwrap_err(),
            OriginError::FaultyScheme
        );
        assert_eq!(
            Origin::new("sub.example.com:8080").unwrap_err(),
            OriginError::FaultyScheme
        );
    }

    #[test]
    fn origin_length_invalidation() {
        let origin = "https://thisisaridiculouslylongurithatshoulddefinitelybeinvalidaccordingtothistest.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijkl.com";
        assert_eq!(
            Origin::new(origin).unwrap_err(),
            OriginError::FaultyUriLength
        )
    }

    #[test]
    fn origin_part_length_invalidation() {
        let origin = "https://www.abcdefghijklmnopqrstuvwxyzabcdefghijklmnoqrstuvwxyzabcdefghijklmnopqrstuvwxyz.com";
        assert_eq!(Origin::new(origin).unwrap_err(), OriginError::InvalidHost)
    }

    #[test]
    fn origin_port_invalidation() {
        assert_eq!(
            Origin::new("http://example.com:abcd").unwrap_err(),
            OriginError::FaultyPort
        )
    }

    #[test]
    fn origin_invalid_ip_port_range_invalidation() {
        // Origin:new with a faulty IP should give OriginError::FaultyPort.
        assert_eq!(
            Origin::new("https://192.168.1.0:80080").unwrap_err(),
            OriginError::FaultyPort
        )
    }

    #[test]
    fn origin_uri_with_invalid_suffix_invalidation() {
        // Origin::new cannot have a path, fragment or query in it and should throw an error.
        assert_eq!(
            Origin::new("https://192.168.1.0#hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://192.168.1.0/hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://192.168.1.0/#hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://192.168.1.0/?q=hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://192.168.1.0?q=hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://example.com/hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://example.com/?q=helloworld").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://example.com?q=helloworld").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://example.com/#hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
        assert_eq!(
            Origin::new("https://example.com#hello").unwrap_err(),
            OriginError::InvalidSuffix
        );
    }

    #[test]
    fn origin_non_existent_host_invalidation() {
        assert_eq!(
            Origin::new("/api/hello").unwrap_err(),
            OriginError::FaultyScheme
        );

        assert!(
            Origin::new("https:///api/hello").is_err() // http::URI gives an InvalidUri InvalidScheme error
        );
    }
}
