macro_rules! status {
    ( $( $name:ident = $message:literal, )* ) => {
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
    SwitchingProtocols  = "101 Switching Protocols",

    OK                  = "200 OK",
    Created             = "201 Created",
    NoContent           = "204 No Content",

    MovedPermanently    = "301 Moved Permanently",
    Found               = "302 Found",
    NotModified         = "304 Not Modifed",
            
    UnprocessableEntity = "422 Unprocessable Entity",
    BadRequest          = "400 Bad Request",
    Unauthorized        = "401 Unauthorized",
    Forbidden           = "403 Forbidden",
    NotFound            = "404 Not Found",

    InternalServerError = "500 Internal Server Error",
    NotImplemented      = "501 Not Implemented",
}

const _: () = {
    impl std::fmt::Debug for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }
};
