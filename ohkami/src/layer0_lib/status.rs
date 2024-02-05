macro_rules! status {
    (
        $(
            $name:ident => $message:literal,
        )*
    ) => {
        #[derive(PartialEq, Clone, Copy)]
        pub enum Status {
            $( $name, )*
        }

        impl Status {
            #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$name => $message, )*
                }
            }
            #[inline(always)] pub(crate) const fn as_bytes(&self) -> &'static [u8] {
                self.as_str().as_bytes()
            }
        }
    };
} status! {
    Continue                    => "100 Continue",
    SwitchingProtocols          => "101 Switching Protocols",
    Processing                  => "102 Processing",
    EarlyHints                  => "103 Early Hints",

    OK                          => "200 OK",
    Created                     => "201 Created",
    Accepted                    => "202 Accepted",
    NonAuthoritativeInformation => "203 Non-Authoritative Information",
    NoContent                   => "204 No Content",
    ResetContent                => "205 Reset Content",
    PartialContent              => "206 Partial Content",
    MultiStatus                 => "207 Multi-Status",
    AlreadyReported             => "208 Already Reported",
    IMUsed                      => "226 IMUsed",

    MultipleChoice              => "300 Multiple Choice",
    MovedPermanently            => "301 Moved Permanently",
    Found                       => "302 Found",
    SeeOther                    => "303 See Other",
    NotModified                 => "304 Not Modifed",
    TemporaryRedirect           => "307 Temporary Redirect",
    PermanentlRedirect          => "308 Permanent Redirect",

    BadRequest                  => "400 Bad Request",
    Unauthorized                => "401 Unauthorized",
    Forbidden                   => "403 Forbidden",
    NotFound                    => "404 Not Found",
    MethodNotAllowed            => "405 Method Not Allowed",
    UnprocessableEntity         => "422 Unprocessable Entity",

    InternalServerError => "500 Internal Server Error",
    NotImplemented      => "501 Not Implemented",
}

const _: () = {
    impl std::fmt::Debug for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
    impl std::fmt::Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
};
