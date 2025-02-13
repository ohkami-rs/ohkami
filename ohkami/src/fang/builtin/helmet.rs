/// based on <https://hono.dev/docs/middleware/builtin/secure-headers>,
/// with removing non-standard or deprecated headers
/// 
/// ## Example
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::Helmet;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((Helmet::default(),
///         "/hello".GET(|| async {"Hello, Helmet!"}),
///     )).howl("localhost:4040").await
/// }
/// ```
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Helmet {
    ContentSecurityPolicy:           Option<CSP>,
    ContentSecurityPolicyReportOnly: Option<CSP>,
    CrossOriginEmbedderPolicy:       &'static str,
    CrossOriginResourcePolicy:       &'static str,
    ReferrerPolicy:                  &'static str,
    StrictTransportSecurity:         &'static str,
    XContentTypeOptions:             &'static str,
    XFrameOptions:                   &'static str,
}
const _: () = {
    impl Default for Helmet {
        fn default() -> Self {
            Helmet {
                ContentSecurityPolicy:           None,
                ContentSecurityPolicyReportOnly: None,
                CrossOriginEmbedderPolicy:       "require-corp",
                CrossOriginResourcePolicy:       "same-origin",
                ReferrerPolicy:                  "no-referrer",
                StrictTransportSecurity:         "max-age=15552000; includeSubDomains",
                XContentTypeOptions:             "nosniff",
                XFrameOptions:                   "SAMEORIGIN",
            }
        }
    }

    #[allow(non_snake_case)]
    impl Helmet {
        /// default: no setting
        pub fn ContentSecurityPolicy(mut self, csp: CSP) -> Self {
            self.ContentSecurityPolicy = Some(csp); self
        }
        /// default: no setting
        pub fn ContentSecurityPolicyReportOnly(mut self, csp: CSP) -> Self {
            self.ContentSecurityPolicyReportOnly = Some(csp); self
        }
        /// default: `"require-corp"`
        pub fn CrossOriginEmbedderPolicy(mut self, CrossOriginEmbedderPolicy: &'static str) -> Self {
            self.CrossOriginEmbedderPolicy = CrossOriginEmbedderPolicy; self
        }
        /// default: `"same-origin"`
        pub fn CrossOriginResourcePolicy(mut self, CrossOriginResourcePolicy: &'static str) -> Self {
            self.CrossOriginResourcePolicy = CrossOriginResourcePolicy; self
        }
        /// default: `"no-referrer"`
        pub fn ReferrerPolicy(mut self, ReferrerPolicy: &'static str) -> Self {
            self.ReferrerPolicy = ReferrerPolicy; self
        }
        /// default: `"max-age=15552000; includeSubDomains"`
        pub fn StrictTransportSecurity(mut self, StrictTransportSecurity: &'static str) -> Self {
            self.StrictTransportSecurity = StrictTransportSecurity; self
        }
        /// default: `"nosniff"`
        pub fn XContentTypeOptions(mut self, XContentTypeOptions: &'static str) -> Self {
            self.XContentTypeOptions = XContentTypeOptions; self
        }
        /// default: `"SAMEORIGIN"`
        pub fn XFrameOptions(mut self, XFrameOptions: &'static str) -> Self {
            self.XFrameOptions = XFrameOptions; self
        }
    }

    impl Helmet {
        pub(self) fn apply(&self, res: &mut crate::Response) {
            let mut h = res.headers.set();
            if let Some(csp) = &self.ContentSecurityPolicy {
                h = h.ContentSecurityPolicy(csp.build());
            }
            if let Some(csp) = &self.ContentSecurityPolicyReportOnly {
                h = h.ContentSecurityPolicyReportOnly(csp.build());
            }
            if !self.CrossOriginEmbedderPolicy.is_empty() {
                h = h.CrossOriginEmbedderPolicy(self.CrossOriginEmbedderPolicy);
            }
            if !self.CrossOriginResourcePolicy.is_empty() {
                h = h.CrossOriginResourcePolicy(self.CrossOriginResourcePolicy);
            }
            if !self.ReferrerPolicy.is_empty() {
                h = h.ReferrerPolicy(self.ReferrerPolicy);
            }
            if !self.StrictTransportSecurity.is_empty() {
                h = h.StrictTransportSecurity(self.StrictTransportSecurity);
            }
            if !self.XContentTypeOptions.is_empty() {
                h = h.XContentTypeOptions(self.XContentTypeOptions);
            }
            if !self.XFrameOptions.is_empty() {
                h.XFrameOptions(self.XFrameOptions);
            }
        }
    }
};

/// based on <https://content-security-policy.com>
/// 
/// ## Example
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::helmet::{Helmet, CSP, sandbox::{allow_forms, allow_same_origin}};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Helmet::default()
///             .ContentSecurityPolicy(CSP {
///                 sandbox: allow_forms | allow_same_origin,
///                 ..Default::default()
///             }),
///     )).howl("localhost:4040").await
/// }
/// ```
#[derive(Clone, Default)]
pub struct CSP {
    pub default_src:               source::SourceList,
    pub script_src:                source::SourceList,
    pub style_src:                 source::SourceList,
    pub img_src:                   source::SourceList,
    pub connect_src:               source::SourceList,
    pub font_src:                  source::SourceList,
    pub object_src:                source::SourceList,
    pub media_src:                 source::SourceList,
    pub frame_src:                 source::SourceList,
    pub sandbox:                   sandbox::Sandbox,
    pub report_uri:                &'static str,
    pub child_src:                 source::SourceList,
    pub form_action:               &'static str,
    pub frame_ancestors:           &'static str,
    pub plugin_types:              &'static str,
    pub base_uri:                  &'static str,
    pub report_to:                 &'static str,
    pub worker_src:                source::SourceList,
    pub manifest_src:              source::SourceList,
    pub prefetch_src:              source::SourceList,
    pub navifate_to:               &'static str,
    pub require_trusted_types_for: &'static str,
    pub trusted_types:             &'static str,
    pub upgrade_insecure_requests: &'static str,
    pub block_all_mixed_content:   &'static str,
}
const _: () = {
    impl CSP {
        pub(self) fn build(&self) -> String {
            let mut result = String::new();

            macro_rules! append {
                ($field:ident build as $policy:literal) => {
                    if !(self.$field.is_empty()) {
                        result.push_str(concat!($policy, " "));
                        result.push_str(&*self.$field.build());
                        result.push(';');
                    }
                };
                ($field:ident as $policy:literal) => {
                    if !(self.$field.is_empty()) {
                        result.push_str(concat!($policy, " "));
                        result.push_str(self.$field);
                        result.push(';');
                    }
                };
            }

            append!(default_src               build as "default-src");
            append!(script_src                build as "script-src");
            append!(style_src                 build as "style-src");
            append!(img_src                   build as "img-src");
            append!(connect_src               build as "connect-src");
            append!(font_src                  build as "font-src");
            append!(object_src                build as "object-src");
            append!(media_src                 build as "media-src");
            append!(frame_src                 build as "frame-src");
            append!(sandbox                   build as "sandbox");
            append!(report_uri                      as "report-uri");
            append!(child_src                 build as "child-src");
            append!(form_action                     as "form-action");
            append!(frame_ancestors                 as "frame-ancestors");
            append!(plugin_types                    as "plugin-types");
            append!(base_uri                        as "base-uri");
            append!(report_to                       as "report-to");
            append!(worker_src                build as "worker-src");
            append!(manifest_src              build as "manifest-src");
            append!(prefetch_src              build as "prefetch_src");
            append!(navifate_to                     as "navifate-to");
            append!(require_trusted_types_for       as "require-trusted-types-for");
            append!(trusted_types                   as "trusted-types");
            append!(upgrade_insecure_requests       as "upgrade-insecure-requests");
            append!(block_all_mixed_content         as "block-all-mixed-content");

            result
        }
    }
};

/// ## Example
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::helmet::{Helmet, CSP, sandbox::{allow_forms, allow_same_origin}};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Helmet::default()
///             .ContentSecurityPolicy(CSP {
///                 sandbox: allow_forms | allow_same_origin,
///                 ..Default::default()
///             }),
///     )).howl("localhost:4040").await
/// }
/// ```
#[allow(non_upper_case_globals)]
pub mod sandbox {
    #[derive(Clone)]
    pub struct Sandbox(u16);

    pub const allow_forms:                    Sandbox = Sandbox(0b0000000001u16);
    pub const allow_same_origin:              Sandbox = Sandbox(0b0000000010u16);
    pub const allow_scripts:                  Sandbox = Sandbox(0b0000000100u16);
    pub const allow_popups:                   Sandbox = Sandbox(0b0000001000u16);
    pub const allow_modals:                   Sandbox = Sandbox(0b0000010000u16);
    pub const allow_orientation_lock:         Sandbox = Sandbox(0b0000100000u16);
    pub const allow_pointer_lock:             Sandbox = Sandbox(0b0001000000u16);
    pub const allow_presentation:             Sandbox = Sandbox(0b0010000000u16);
    pub const allow_popups_to_escape_sandbox: Sandbox = Sandbox(0b0100000000u16);
    pub const allow_top_navigation:           Sandbox = Sandbox(0b1000000000u16);

    impl std::ops::BitOr for Sandbox {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(self.0 | rhs.0)
        }
    }

    impl Default for Sandbox {
        fn default() -> Self {
            Self(0b0000000000u16)
        }
    }

    impl Sandbox {
        pub(super) const fn is_empty(&self) -> bool {
            self.0 == 0b0000000000u16
        }

        pub(super) fn build(&self) -> String {
            let mut result = String::new();
            if self.0 & allow_forms.0 != 0                    {result.push_str(" allow-forms");}
            if self.0 & allow_same_origin.0 != 0              {result.push_str(" allow-same-origin");}
            if self.0 & allow_scripts.0 != 0                  {result.push_str(" allow-scripts");}
            if self.0 & allow_popups.0 != 0                   {result.push_str(" allow-popups");}
            if self.0 & allow_modals.0 != 0                   {result.push_str(" allow-modals");}
            if self.0 & allow_orientation_lock.0 != 0         {result.push_str(" allow-orientation-lock");}
            if self.0 & allow_pointer_lock.0 != 0             {result.push_str(" allow-pointer-lock");}
            if self.0 & allow_presentation.0 != 0             {result.push_str(" allow-presentation");}
            if self.0 & allow_popups_to_escape_sandbox.0 != 0 {result.push_str(" allow-popups-to-escape-sandbox");}
            if self.0 & allow_top_navigation.0 != 0           {result.push_str(" allow-top-navigation");}
            result
        }
    }
}

/// ## Example
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::helmet::{Helmet, CSP, source::{self_origin, data}};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Helmet::default()
///             .ContentSecurityPolicy(CSP {
///                 script_src: self_origin | data,
///                 ..Default::default()
///             }),
///         "/hello".GET(|| async {"Hello, helmet!"})
///     )).howl("localhost:3000").await
/// }
/// ```
#[allow(non_upper_case_globals)]
pub mod source {
    #[derive(Clone, Default)]
    pub struct SourceList {
        this: std::borrow::Cow<'static, str>,
        list: Vec<std::borrow::Cow<'static, str>>,
    }

    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub enum Source {
        any,
        data,
        https,
        none,
        self_origin,
        strict_dynamic,
        unsafe_inline,
        unsafe_eval,
        unsafe_hashes,
        domain(&'static str),
        sha256(String),
        sha384(String),
        sha512(String),
        nonce(String),
    }
    impl Source {
        const fn build_const(&self) -> std::borrow::Cow<'static, str> {
            match self {
                Self::any            => std::borrow::Cow::Borrowed("*"),
                Self::data           => std::borrow::Cow::Borrowed("data:"),
                Self::https          => std::borrow::Cow::Borrowed("https:"),
                Self::none           => std::borrow::Cow::Borrowed("'none'"),
                Self::self_origin    => std::borrow::Cow::Borrowed("'self'"),
                Self::strict_dynamic => std::borrow::Cow::Borrowed("'strict-dynamic'"),
                Self::unsafe_inline  => std::borrow::Cow::Borrowed("'unsafe-inline'"),
                Self::unsafe_eval    => std::borrow::Cow::Borrowed("'unsafe-eval'"),
                Self::unsafe_hashes  => std::borrow::Cow::Borrowed("'unsafe-hashes'"),
                Self::domain(s)      => std::borrow::Cow::Borrowed(*s),
                Self::sha256(_) => unreachable!(),
                Self::sha384(_) => unreachable!(),
                Self::sha512(_) => unreachable!(),
                Self::nonce(_)  => unreachable!(),
            }
        }
        fn build_hash(&self) -> std::borrow::Cow<'static, str> {
            match self {
                Self::any            => unreachable!(),
                Self::data           => unreachable!(),
                Self::https          => unreachable!(),
                Self::none           => unreachable!(),
                Self::self_origin    => unreachable!(),
                Self::strict_dynamic => unreachable!(),
                Self::unsafe_inline  => unreachable!(),
                Self::unsafe_eval    => unreachable!(),
                Self::unsafe_hashes  => unreachable!(),
                Self::domain(_)      => unreachable!(),
                Self::sha256(s) => std::borrow::Cow::Owned(format!("'sha256-{s}'")),
                Self::sha384(s) => std::borrow::Cow::Owned(format!("'sha384-{s}'")),
                Self::sha512(s) => std::borrow::Cow::Owned(format!("'sha512-{s}'")),
                Self::nonce(s)  => std::borrow::Cow::Owned(format!("'nonce-{s}'")),
            }
        }
    }

    macro_rules! this {
        (const $src:expr) => {SourceList { this: $src.build_const(), list: Vec::new() }};
        (hash $src:expr) => {SourceList { this: $src.build_hash(), list: Vec::new() }};
    }
    pub const any:            SourceList = this!(const Source::any);
    pub const data:           SourceList = this!(const Source::data);
    pub const https:          SourceList = this!(const Source::https);
    pub const none:           SourceList = this!(const Source::none);
    pub const self_origin:    SourceList = this!(const Source::self_origin);
    pub const strict_dynamic: SourceList = this!(const Source::strict_dynamic);
    pub const unsafe_inline:  SourceList = this!(const Source::unsafe_inline);
    pub const unsafe_eval:    SourceList = this!(const Source::unsafe_eval);
    pub const unsafe_hashes:  SourceList = this!(const Source::unsafe_hashes);
    pub fn domain(domain: &'static str) -> SourceList {this!(const Source::domain(domain))}
    pub fn sha256(sha256: String) -> SourceList {this!(hash Source::sha256(sha256))}
    pub fn sha384(sha384: String) -> SourceList {this!(hash Source::sha384(sha384))}
    pub fn sha512(sha512: String) -> SourceList {this!(hash Source::sha512(sha512))}
    pub fn nonce(nonce:  String) -> SourceList {this!(hash Source::nonce(nonce))}

    impl std::ops::BitOr for SourceList {
        type Output = Self;

        fn bitor(mut self, rhs: Self) -> Self::Output {
            self.list.push(rhs.this);
            self.list.extend(rhs.list);
            self
        }
    }

    impl SourceList {
        pub(super) fn is_empty(&self) -> bool {
            self.this.is_empty()
        }

        pub(super) fn build(&self) -> String {
            let mut result = String::from(&*self.this);
            if !self.list.is_empty() {
                for s in &self.list {
                    result.push(' ');
                    result.push_str(&*s);
                }
            }
            result
        }
    }
}

const _: () = {
    use crate::{Request, Response, Fang, FangProc};
    use std::sync::OnceLock;

    impl<I: FangProc> Fang<I> for Helmet {
        type Proc = HelmetProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            static SET_HEADERS: OnceLock<Box<dyn Fn(&mut Response) + Send + Sync>> = OnceLock::new();

            let set_headers = SET_HEADERS.get_or_init(|| {
                /* clone **only once** */
                let helmet = self.clone();
                Box::new(move |res: &mut Response| {helmet.apply(res)})
            });

            HelmetProc { inner, set_headers }
        }
    }

    pub struct HelmetProc<I> {
        set_headers: &'static (dyn Fn(&mut Response) + Send + Sync),
        inner: I,
    }

    impl<I: FangProc> FangProc for HelmetProc<I> {
        #[inline]
        async fn bite<'f>(&'f self, req: &'f mut Request) -> Response {
            let mut res = self.inner.bite(req).await;
            (self.set_headers)(&mut res);
            res
        }
    }
};




#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::testing::*;
    use std::collections::HashSet;

    #[test]
    fn test_helmet_set_headers() {
        let t = Ohkami::new((
            Helmet::default(),
            "/hello".GET(|| async {"Hello, helmet!"}),
        )).test();

        crate::__rt__::testing::block_on(async {
            /* matched case */
            {
                let req = TestRequest::GET("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text().unwrap(), "Hello, helmet!");
                assert_eq!(res.headers().collect::<HashSet<_>>(), HashSet::from_iter([
                    ("Cross-Origin-Embedder-Policy", "require-corp"),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                    ("Referrer-Policy", "no-referrer"),
                    ("Strict-Transport-Security", "max-age=15552000; includeSubDomains"),
                    ("X-Content-Type-Options", "nosniff"),
                    ("X-Frame-Options", "SAMEORIGIN"),
                    
                    ("Content-Type", "text/plain; charset=UTF-8"),
                    ("Content-Length", "14"),
                ]));
            }

            /* any Not Found cases */
            {
                let req = TestRequest::GET("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status().code(), 404);
                assert_eq!(res.text(), None);
                assert_eq!(res.headers().collect::<HashSet<_>>(), HashSet::from_iter([
                    ("Cross-Origin-Embedder-Policy", "require-corp"),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                    ("Referrer-Policy", "no-referrer"),
                    ("Strict-Transport-Security", "max-age=15552000; includeSubDomains"),
                    ("X-Content-Type-Options", "nosniff"),
                    ("X-Frame-Options", "SAMEORIGIN"),
                ]));
            }
            {
                let req = TestRequest::POST("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status().code(), 404);
                assert_eq!(res.text(), None);
                assert_eq!(res.headers().collect::<HashSet<_>>(), HashSet::from_iter([
                    ("Cross-Origin-Embedder-Policy", "require-corp"),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                    ("Referrer-Policy", "no-referrer"),
                    ("Strict-Transport-Security", "max-age=15552000; includeSubDomains"),
                    ("X-Content-Type-Options", "nosniff"),
                    ("X-Frame-Options", "SAMEORIGIN"),
                ]));
            }
            {
                let req = TestRequest::DELETE("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status().code(), 404);
                assert_eq!(res.text(), None);
                assert_eq!(res.headers().collect::<HashSet<_>>(), HashSet::from_iter([
                    ("Cross-Origin-Embedder-Policy", "require-corp"),
                    ("Cross-Origin-Resource-Policy", "same-origin"),
                    ("Referrer-Policy", "no-referrer"),
                    ("Strict-Transport-Security", "max-age=15552000; includeSubDomains"),
                    ("X-Content-Type-Options", "nosniff"),
                    ("X-Frame-Options", "SAMEORIGIN"),
                ]));
            }
        });
    }
}
