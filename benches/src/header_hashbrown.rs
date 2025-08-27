use ohkami_lib::{Slice, CowSlice};

type DefaultHasher = super::request_headers::headerhashmap::HeaderHasher;//rustc_hash::FxHasher;

#[inline]
pub fn hash(key: &[u8]) -> u64 {
    use std::hash::Hasher;

    let mut h = DefaultHasher::default();
    h.write(key);
    h.finish()
}


pub struct HeaderHashBrown<const TRACK_SIZE: bool = true> {
    table: table::HeaderHashBrownTable,
    size:  usize,
}

enum HeaderValue {
    Bytes(Vec<u8>),
    Slice(Slice),
    #[allow(unused)]
    Usize(usize),
} const _: () = {
    impl HeaderValue {
        #[inline]
        fn as_bytes(&self) -> &[u8] {
            match self {
                Self::Bytes(v) => v.as_slice(),
                Self::Slice(s) => unsafe {s.as_bytes()},
                Self::Usize(_) => unimplemented!()
            }
        }
    }

    impl From<CowSlice> for HeaderValue {
        #[inline]
        fn from(value: CowSlice) -> Self {
            match value {
                CowSlice::Own(o) => Self::Bytes(o.into()),
                CowSlice::Ref(r) => Self::Slice(r),
            }
        }
    }
};

impl<const TRACK_SIZE: bool> HeaderHashBrown<TRACK_SIZE> {
    pub fn new() -> Self {
        Self {
            table: table::HeaderHashBrownTable::new(),
            size:  "\r\n".len(),
        }
    }

    #[inline]
    pub fn insert_standard(&mut self,
        standard: StandardHeader,
        value:    CowSlice,
    ) -> &mut Self {
        if TRACK_SIZE {
            self.size += standard.as_str().len() + ": ".len() + value.len() + "\r\n".len()
        }
        let key = Slice::from_bytes(standard.as_str().as_bytes());
        unsafe {self.table.insert_known(standard.hash(), key, value.into())}
        self
    }
    #[inline]
    pub fn insert(&mut self,
        key:   &'static str,
        value: CowSlice,
    ) -> &mut Self {
        if TRACK_SIZE {
            self.size += key.len() + ": ".len() + value.len() + "\r\n".len()
        }
        let key   = Slice::from_bytes(key.as_bytes());
        self.table.insert(key, value.into());
        self
    }

    #[inline]
    pub fn insert_standard_from_reqbytes(&mut self,
        standard: StandardHeader,
        value:    &[u8],
    ) -> &mut Self {
        if TRACK_SIZE {
            self.size += standard.as_str().len() + ": ".len() + value.len() + "\r\n".len()
        }
        let key   = Slice::from_bytes(standard.as_str().as_bytes());
        let value = CowSlice::Ref(Slice::from_bytes(value));
        unsafe {self.table.insert_known(standard.hash(), key, value.into())}
        self
    }
    #[inline]
    pub fn insert_from_reqbytes(&mut self,
        key:   &[u8],
        value: &[u8],
    ) -> &mut Self {
        if TRACK_SIZE {
            self.size += key.len() + ": ".len() + value.len() + "\r\n".len()
        }
        let key   = Slice::from_bytes(key);
        let value = CowSlice::Ref(Slice::from_bytes(value));
        self.table.insert(key, value.into());
        self
    }

    #[inline]
    pub fn remove_standard(&mut self,
        standard: StandardHeader,
    ) -> &mut Self {
        let key   = Slice::from_bytes(standard.as_str().as_bytes());
        let value = unsafe {self.table.remove_known(standard.hash(), &key)};
        if TRACK_SIZE {
            if let Some(v) = value {
                self.size -= standard.as_str().len() + ": ".len() + v.as_bytes().len() + "\r\n".len()
            }
        }
        self
    }
    #[inline]
    pub fn remove(&mut self,
        key: &'static str
    ) -> &mut Self {
        let key = Slice::from_bytes(key.as_bytes());
        let value = self.table.remove(&key);
        if TRACK_SIZE {
            if let Some(v) = value {
                self.size -= unsafe {key.as_bytes()}.len() + ": ".len() + v.as_bytes().len() + "\r\n".len()
            }
        }
        self
    }
}

impl HeaderHashBrown<false> {
    #[inline]
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        self.table.write_to(buf)
    }
}
impl HeaderHashBrown<true> {
    #[inline]
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.reserve(self.size);
        unsafe {self.table.write_unchecked_to(buf)}
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
    Accept = "Accept" as 8956897560123365965//16433268118574137039
    AcceptEncoding = "Accept-Encoding" as 9008826061000250594//2625511035195335676
    AcceptLanguage = "Accept-Language" as 263798198000172577//4857106753043711123
    AcceptRanges = "Accept-Ranges" as 18342038782328862764//12598308797832930634
    AccessControlAllowCredentials = "Access-Control-Allow-Credentials" as 9569548883698504606//9116155820374356126
    AccessControlAllowHeaders = "Access-Control-Allow-Headers" as 14084078790202211956//8814696385715034476
    AccessControlAllowMethods = "Access-Control-Allow-Methods" as 5070491789143864837//5462557967219305584
    AccessControlAllowOrigin = "Access-Control-Allow-Origin" as 10178641993106032301//5378217592900298305
    AccessControlExposeHeaders = "Access-Control-Expose-Headers" as 16649258875025620622//13325522807785516598
    AccessControlMaxAge = "Access-Control-Max-Age" as 6947267048838179798//4432901313932580618
    AccessControlRequestHeaders = "Access-Control-Request-Headers" as 14077911138246316357//16301979022674213810
    AccessControlRequestMethod = "Access-Control-Request-Method" as 1511226599830663409//11634788784195468787
    Age = "Age" as 12164395943281393619//10870321372244433485
    Allow = "Allow" as 101864317638997375//3848169699148495437
    AltSvc = "Alt-Svc" as 163385565780702487//5918467845764560387
    Authorization = "Authorization" as 17342828658748765377//12196702939659785452
    CacheControl = "Cache-Control" as 12700798416643114791//11800019523689531337
    CacheStatus = "Cache-Status" as 4830815728491215736//18085679534749337128
    CDNCacheControl = "CDN-Cache-Control" as 15067968555331201001//4331749271142744016
    Connection = "Connection" as 12632990663184834470//16783757005565428516
    ContentDisposition = "Content-Disposition" as 1390085196203246353//15172909992608599841
    ContentEcoding = "Content-Ecoding" as 1761535790260946701//16593443043870130009
    ContentLanguage = "Content-Language" as 1401788212079168976//16735614920345560642
    ContentLength = "Content-Length" as 14843332951706164276//14334207866575450264
    ContentLocation = "Content-Location" as 6809348355172982736//3944620592910008386
    ContentRange = "Content-Range" as 6591774876766068439//11588459248563791643
    ContentSecurityPolicy = "Content-Security-Policy" as 7848988030993024328//5108162438765258431
    ContentSecurityPolicyReportOnly = "Content-Security-Policy-Report-Only" as 8225512036531862485//1939240664108222842
    ContentType = "Content-Type" as 1539870117023715624//3996025485011955786
    Cookie = "Cookie" as 12510759127542743569//17962636191368536035
    Date = "Date" as 2562613028085471028//17579805628842460308
    ETag = "ETag" as 14205462794407424201//18254449783657381417
    Expect = "Expect" as 3319114356378929571//9494374193384502225
    Expires = "Expires" as 14717995381802874822//4291902732285004317
    Forwarded = "Forwarded" as 12510709791974329387//7787083747984806917
    From = "From" as 3435607823061342//15020628208580050622
    Host = "Host" as 3868342997265016712//438791524312454376
    IfMatch = "If-Match" as 758385572210193693//17728942688211657341
    IfModifiedSince = "If-Modified-Since" as 15420386658409231737//6352457413450827350
    IfNoneMatch = "If-None-Match" as 8766751325359657529//3333932262875561685
    IfRange = "If-Range" as 4422112474835105053//2945925517127017085
    IfUnmodifiedSince = "If-Unmodified-Since" as 14842325997600933810//7522477305903254470
    Link = "Link" as 6207054705583559644//2777503232630997308
    Location = "Location" as 1632295297794314716//16649487898551303996
    MaxForwards = "Max-Forwards" as 7426081339672782312//10752408927369271123
    Origin = "Origin" as 5691687282345579944//14882833577272632186
    ProxyAuthenticate = "Proxy-Authenticate" as 14130340937869200619//1820963910701534218
    ProxyAuthorization = "Proxy-Authorization" as 4007097940433767016//12714354196972183062
    Range = "Range" as 13591622306488845170//10582771998975603868
    Referer = "Referer" as 61951474100055896//5839330224843872351
    ReferrerPolicy = "Referrer-Policy" as 1327666139445013389//18395389122136826733
    Refresh = "Refresh" as 7953246256297787639//15850643017965868815
    RetryAfter = "Retry-After" as 11304873063226856260//13276509559803940695
    SecWebSocketAccept = "Sec-WebSocket-Accept" as 5952345478380611784//10946272471545366737
    SecWebSocketExtensions = "Sec-WebSocket-Extensions" as 12765399274657545454//17103059385744334201
    SecWebSocketKey = "Sec-WebSocket-Key" as 11097846330773677699//13420602090516222027
    SecWebSocketProtocol = "Sec-WebSocket-Protocol" as 16408706031545691252//11040576895242091634
    SecWebSocketVersion = "Sec-WebSocket-Version" as 11714057070643420239//5330225619909672710
    Server = "Server" as 2419935139755271097//11765940313756672059
    SetCookie = "Set-Cookie" as 5506158778252165240//3623682265152868430
    StrictTransportSecurity = "Strict-Transport-Security" as 828070379554355266//13089560602798786294
    TE = "TE" as 2663045123408499844//6712032112658457060
    Trailer = "Trailer" as 7062438620934618372//15190164523930466561
    TransferEncoding = "Transfer-Encoding" as 7495137910697819204//8612619927895477042
    Upgrade = "Upgrade" as 11782373995271654455//3830257985504030272
    UpgradeInsecureRequests = "Upgrade-Insecure-Requests" as 11536776535922301664//12060850129311366976
    UserAgent = "User-Agent" as 9952940223324636988//3519543940131721058
    Vary = "Vary" as 12247033862576493998//8817482389623931662
    Via = "Via" as 1872335714014322414//7229469575117716336
    WWWAuthenticate = "WWW-Authenticate" as 8830284111271749131
    XContentTypeOptions = "X-Content-Type-Options" as 10317259392692853873//17298563304118097688
    XFrameOptions = "X-Frame-Options" as 15858069221280842781//4381497337076230406
}


mod table {
    use std::{hash::Hasher, marker::PhantomData};
    use ohkami_lib::Slice;
    use hashbrown::raw::RawTable;
    use super::{DefaultHasher, hash, HeaderValue};

    pub struct HeaderHashBrownTable<H: Hasher + Default = DefaultHasher> {
        table:  RawTable<(Slice, HeaderValue)>,
        hasher: PhantomData<H>,
    }

    impl<H: Hasher + Default> HeaderHashBrownTable<H> {
        pub fn new() -> Self {
            Self {
                table:  RawTable::with_capacity(16),
                hasher: PhantomData
            }
        }

        #[inline(always)]
        const fn hasher() -> impl Fn(&(Slice, HeaderValue))->u64 {
            |(k, _)| hash(unsafe {k.as_bytes()})
        }
        #[inline(always)]
        const fn equivalent(key: &Slice) -> impl Fn(&(Slice, HeaderValue))->bool + '_ {
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
        pub fn insert(&mut self, key: Slice, value: HeaderValue) {
            unsafe { self.insert_known(hash(key.as_bytes()), key, value) }
        }
        /// SAFETY: `hash` BE the hash value of `key`
        #[inline]
        pub unsafe fn insert_known(&mut self, hash: u64, key: Slice, value: HeaderValue) {
            match self.table.find_or_find_insert_slot(hash, Self::equivalent(&key), Self::hasher()) {
                Ok(bucket) => unsafe { bucket.as_mut().1 = value }
                Err(slot)  => unsafe { self.table.insert_in_slot(hash, slot, (key, value)); }
            }
        }

        #[inline]
        pub fn remove(&mut self, key: &Slice) -> Option<HeaderValue> {
            unsafe { self.remove_known(hash(key.as_bytes()), key) }
        }
        /// SAFETY: `hash` BE the hash value of `key`
        #[inline]
        pub unsafe fn remove_known(&mut self, hash: u64, key: &Slice) -> Option<HeaderValue> {
            match self.table.remove_entry(hash, Self::equivalent(key)) {
                Some((_, v)) => Some(v),
                None => None
            }
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

        /// SAFETY: `buf` has enough remaining capacity
        #[inline]
        pub unsafe fn write_unchecked_to(&self, buf: &mut Vec<u8>) {
            macro_rules! push_unchecked {
                ($bytes:expr) => {
                    unsafe {
                        let (buf_len, bytes_len) = (buf.len(), $bytes.len());
                        std::ptr::copy_nonoverlapping(
                            $bytes.as_ptr(),
                            buf.as_mut_ptr().add(buf_len),
                            bytes_len
                        );
                        buf.set_len(
                            buf_len + bytes_len
                        );
                    }
                };
            }
            for bucket in unsafe {self.table.iter()} {
                let (k, v) = unsafe {bucket.as_ref()};
                push_unchecked!(k.as_bytes());
                push_unchecked!(b": ");
                push_unchecked!(v.as_bytes());
                push_unchecked!(b"\r\n");
            }
            push_unchecked!(b"\r\n");
        }
    }
}
