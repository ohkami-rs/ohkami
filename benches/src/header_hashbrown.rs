use ohkami_lib::{Slice, CowSlice};

type DefaultHasher = super::request_headers::headerhashmap::HeaderHasher;//rustc_hash::FxHasher;

#[inline]
pub fn hash(key: &[u8]) -> u64 {
    use std::hash::Hasher;

    let mut h = DefaultHasher::default();
    h.write(key);
    h.finish()
}


pub struct HeaderHashBrown(
    table::HeaderHashBrownTable
);

impl HeaderHashBrown {
    pub fn new() -> Self {
        Self(table::HeaderHashBrownTable::new())
    }

    #[inline]
    pub fn insert_standard(&mut self,
        standard: StandardHeader,
        value:    CowSlice,
    ) -> &mut Self {
        let key   = Slice::from_bytes(standard.as_str().as_bytes());
        unsafe {self.0.insert_known(
            standard.hash(),
            key, value
        )}
        self
    }
    #[inline]
    pub fn insert(&mut self,
        key:   &'static str,
        value: CowSlice,
    ) -> &mut Self {
        let key   = Slice::from_bytes(key.as_bytes());
        self.0.insert(key, value);
        self
    }

    #[inline]
    pub fn insert_standard_from_reqbytes(&mut self,
        standard: StandardHeader,
        value:    &[u8],
    ) -> &mut Self {
        let key   = Slice::from_bytes(standard.as_str().as_bytes());
        let value = CowSlice::Ref(Slice::from_bytes(value));
        unsafe {self.0.insert_known(
            standard.hash(),
            key, value
        )}
        self
    }
    #[inline]
    pub fn insert_from_reqbytes(&mut self,
        key:   &[u8],
        value: &[u8],
    ) -> &mut Self {
        let key   = Slice::from_bytes(key);
        let value = CowSlice::Ref(Slice::from_bytes(value));
        self.0.insert(key, value);
        self
    }

    #[inline]
    pub fn remove_standard(&mut self,
        standard: StandardHeader,
    ) -> &mut Self {
        let key = Slice::from_bytes(standard.as_str().as_bytes());
        unsafe {self.0.remove_known(
            standard.hash(),
            &key
        )}
        self
    }
    #[inline]
    pub fn remove(&mut self,
        key: &'static str
    ) -> &mut Self {
        let key = Slice::from_bytes(key.as_bytes());
        self.0.remove(&key);
        self
    }

    #[inline]
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        self.0.write_to(buf)
    }
}

macro_rules! StandardHeader {
    ( $( $variant:ident = $name:literal as $hash:literal )* ) => {
        pub enum StandardHeader {
            $( $variant ),*
        }

        impl StandardHeader {
            #[inline(always)]
            const fn as_str(&self) -> &'static str {
                match self {
                    $( Self::$variant => $name ),*
                }
            }

            #[inline(always)]
            const fn hash(&self) -> u64 {
                match self {
                    $( Self::$variant => $hash ),*
                }
            }

            #[inline]
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match hash(bytes) {
                    $( $hash => Some(Self::$variant), )*
                    _ => None
                }
            }
        }

        #[cfg(test)]
        #[test] fn correct_hash() {
            $(
                let s = StandardHeader::$variant;
                assert_eq!(
                    s.hash(),
                    hash(s.as_str().as_bytes())
                );
            )*
        }
    };
} StandardHeader! {
    Accept = "Accept" as 16433268118574137039
    AcceptEncoding = "Accept-Encoding" as 2625511035195335676
    AcceptLanguage = "Accept-Language" as 4857106753043711123
    AcceptRanges = "Accept-Ranges" as 12598308797832930634
    AccessControlAllowCredentials = "Access-Control-Allow-Credentials" as 9116155820374356126
    AccessControlAllowHeaders = "Access-Control-Allow-Headers" as 8814696385715034476
    AccessControlAllowMethods = "Access-Control-Allow-Methods" as 5462557967219305584
    AccessControlAllowOrigin = "Access-Control-Allow-Origin" as 5378217592900298305
    AccessControlExposeHeaders = "Access-Control-Expose-Headers" as 13325522807785516598
    AccessControlMaxAge = "Access-Control-Max-Age" as 4432901313932580618
    AccessControlRequestHeaders = "Access-Control-Request-Headers" as 16301979022674213810
    AccessControlRequestMethod = "Access-Control-Request-Method" as 11634788784195468787
    Age = "Age" as 10870321372244433485
    Allow = "Allow" as 3848169699148495437
    AltSvc = "Alt-Svc" as 5918467845764560387
    Authorization = "Authorization" as 12196702939659785452
    CacheControl = "Cache-Control" as 11800019523689531337
    CacheStatus = "Cache-Status" as 18085679534749337128
    CDNCacheControl = "CDN-Cache-Control" as 4331749271142744016
    Connection = "Connection" as 16783757005565428516
    ContentDisposition = "Content-Disposition" as 15172909992608599841
    ContentEcoding = "Content-Ecoding" as 16593443043870130009
    ContentLanguage = "Content-Language" as 16735614920345560642
    ContentLength = "Content-Length" as 14334207866575450264
    ContentLocation = "Content-Location" as 3944620592910008386
    ContentRange = "Content-Range" as 11588459248563791643
    ContentSecurityPolicy = "Content-Security-Policy" as 5108162438765258431
    ContentSecurityPolicyReportOnly = "Content-Security-Policy-Report-Only" as 1939240664108222842
    ContentType = "Content-Type" as 3996025485011955786
    Cookie = "Cookie" as 17962636191368536035
    Date = "Date" as 17579805628842460308
    ETag = "ETag" as 18254449783657381417
    Expect = "Expect" as 9494374193384502225
    Expires = "Expires" as 4291902732285004317
    Forwarded = "Forwarded" as 7787083747984806917
    From = "From" as 15020628208580050622
    Host = "Host" as 438791524312454376
    IfMatch = "If-Match" as 17728942688211657341
    IfModifiedSince = "If-Modified-Since" as 6352457413450827350
    IfNoneMatch = "If-None-Match" as 3333932262875561685
    IfRange = "If-Range" as 2945925517127017085
    IfUnmodifiedSince = "If-Unmodified-Since" as 7522477305903254470
    Link = "Link" as 2777503232630997308
    Location = "Location" as 16649487898551303996
    MaxForwards = "Max-Forwards" as 10752408927369271123
    Origin = "Origin" as 14882833577272632186
    ProxyAuthenticate = "Proxy-Authenticate" as 1820963910701534218
    ProxyAuthorization = "Proxy-Authorization" as 12714354196972183062
    Range = "Range" as 10582771998975603868
    Referer = "Referer" as 5839330224843872351
    ReferrerPolicy = "Referrer-Policy" as 18395389122136826733
    Refresh = "Refresh" as 15850643017965868815
    RetryAfter = "Retry-After" as 13276509559803940695
    SecWebSocketAccept = "Sec-WebSocket-Accept" as 10946272471545366737
    SecWebSocketExtensions = "Sec-WebSocket-Extensions" as 17103059385744334201
    SecWebSocketKey = "Sec-WebSocket-Key" as 13420602090516222027
    SecWebSocketProtocol = "Sec-WebSocket-Protocol" as 11040576895242091634
    SecWebSocketVersion = "Sec-WebSocket-Version" as 5330225619909672710
    Server = "Server" as 11765940313756672059
    SetCookie = "SetCookie" as 3623682265152868430
    StrictTransportSecurity = "Strict-Transport-Security" as 13089560602798786294
    TE = "TE" as 6712032112658457060
    Trailer = "Trailer" as 15190164523930466561
    TransferEncoding = "Transfer-Encoding" as 8612619927895477042
    Upgrade = "Upgrade" as 3830257985504030272
    UpgradeInsecureRequests = "Upgrade-Insecure-Requests" as 12060850129311366976
    UserAgent = "User-Agent" as 3519543940131721058
    Vary = "Vary" as 8817482389623931662
    Via = "Via" as 7229469575117716336
    XContentTypeOptions = "X-Content-Type-Options" as 17298563304118097688
    XFrameOptions = "X-Frame-Options" as 4381497337076230406
}


mod table {
    use std::{hash::Hasher, marker::PhantomData};
    use ohkami_lib::{Slice, CowSlice};
    use hashbrown::raw::RawTable;
    use super::{DefaultHasher, hash};

    pub struct HeaderHashBrownTable<H: Hasher + Default = DefaultHasher> {
        table:  RawTable<(Slice, CowSlice)>,
        hasher: PhantomData<H>,
    }

    impl<H: Hasher + Default> HeaderHashBrownTable<H> {
        pub fn new() -> Self {
            Self {
                table:  RawTable::with_capacity(32),
                hasher: PhantomData
            }
        }

        #[inline(always)]
        fn hasher() -> impl Fn(&(Slice, CowSlice))->u64 {
            |(k, _)| hash(unsafe {k.as_bytes()})
        }
        #[inline(always)]
        fn equivalent(key: &Slice) -> impl Fn(&(Slice, CowSlice))->bool + '_ {
            move |(k, _)| k == key
        }
    }
    
    impl<H: Hasher + Default> HeaderHashBrownTable<H> {
        // #[inline]
        // pub fn get(&self, key: &Slice) -> Option<&CowSlice> {
        //     self.get_kown(Self::hash(key), key)
        // }
        // #[inline]
        // pub fn get_kown(&self, hash: u64, key: &Slice) -> Option<&CowSlice> {
        //     match self.table.get(hash, Self::equivalent(key)) {
        //         Some((_, value)) => Some(value),
        //         None             => None,
        //     }
        // }

        #[inline]
        pub fn insert(&mut self, key: Slice, value: CowSlice) {
            unsafe { self.insert_known(hash(key.as_bytes()), key, value) }
        }
        /// SAFETY: `hash` BE the hash value of `key`
        #[inline]
        pub unsafe fn insert_known(&mut self, hash: u64, key: Slice, value: CowSlice) {
            match self.table.find_or_find_insert_slot(hash, Self::equivalent(&key), Self::hasher()) {
                Ok(bucket) => unsafe { bucket.as_mut().1 = value }
                Err(slot)  => unsafe { self.table.insert_in_slot(hash, slot, (key, value)); }
            }
        }

        #[inline]
        pub fn remove(&mut self, key: &Slice) {
            unsafe { self.remove_known(hash(key.as_bytes()), key) }
        }
        /// SAFETY: `hash` BE the hash value of `key`
        #[inline]
        pub unsafe fn remove_known(&mut self, hash: u64, key: &Slice) {
            self.table.remove_entry(hash, Self::equivalent(key));
        }

        #[inline]
        pub fn write_to(&self, buf: &mut Vec<u8>) {
            unsafe {
                for bucket in self.table.iter() {
                    let (k, v) = bucket.as_ref();
                    buf.extend_from_slice(k.as_bytes());
                    buf.extend_from_slice(b": ");
                    buf.extend_from_slice(v.as_bytes());
                    buf.extend_from_slice(b"\r\n");
                }
            }
            buf.extend_from_slice(b"\r\n");
        }
    }
}
