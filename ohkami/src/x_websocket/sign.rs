mod sha1;   pub use sha1::  {Sha1, SIZE as SHA1_SIZE};
mod base64; pub use base64::{Base64};

#[cfg(test)] mod sign_test {
    use super::*;

    #[test] fn test_sha1() {// https://github.com/golang/go/blob/master/src/crypto/sha1/sha1_test.go

    }

    #[test] fn test_base64() {// https://github.com/golang/go/blob/master/src/encoding/base64/base64_test.go
        // RFC 3548 examples
	    assert_eq!(Base64::<6>::encode(*b"\x14\xfb\x9c\x03\xd9\x7e"), "FPucA9l+");
	    assert_eq!(Base64::<5>::encode(*b"\x14\xfb\x9c\x03\xd9"),     "FPucA9k=");
	    assert_eq!(Base64::<4>::encode(*b"\x14\xfb\x9c\x03"),         "FPucAw==");
        
        // RFC 4648 examples
	    assert_eq!(Base64::<0>::encode(*b""),       "");
	    assert_eq!(Base64::<1>::encode(*b"f"),      "Zg==");
	    assert_eq!(Base64::<2>::encode(*b"fo"),     "Zm8=");
	    assert_eq!(Base64::<3>::encode(*b"foo"),    "Zm9v");
	    assert_eq!(Base64::<4>::encode(*b"foob"),   "Zm9vYg==");
	    assert_eq!(Base64::<5>::encode(*b"fooba"),  "Zm9vYmE=");
	    assert_eq!(Base64::<6>::encode(*b"foobar"), "Zm9vYmFy");
        
        // Wikipedia examples
	    assert_eq!(Base64::<5>::encode(*b"sure."),    "c3VyZS4=");
	    assert_eq!(Base64::<4>::encode(*b"sure"),     "c3VyZQ==");
	    assert_eq!(Base64::<3>::encode(*b"sur"),      "c3Vy");
	    assert_eq!(Base64::<2>::encode(*b"su"),       "c3U=");
	    assert_eq!(Base64::<8>::encode(*b"leasure."), "bGVhc3VyZS4=");
	    assert_eq!(Base64::<7>::encode(*b"easure."),  "ZWFzdXJlLg==");
	    assert_eq!(Base64::<6>::encode(*b"asure."),   "YXN1cmUu");
	    assert_eq!(Base64::<5>::encode(*b"sure."),    "c3VyZS4=");
    }
}
