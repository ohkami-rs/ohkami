/// based on <https://hono.dev/docs/middleware/builtin/secure-headers>,
/// with removing non-standard or deprecated headers
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct Helmet {
    pub ContentSecurityPolicy:           Option<CSP>,
    pub ContentSecurityPolicyReportOnly: Option<CSP>,
    pub CrossOriginEmbedderPolicy:       &'static str,
    pub CrossOriginResourcePolicy:       &'static str,
    pub ReferrerPolicy:                  &'static str,
    pub StrictTransportSecurity:         &'static str,
    pub XContentTypeOptions:             &'static str,
    pub XFrameOptions:                   &'static str,
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
#[derive(Clone, Default)]
pub struct CSP {
    pub default_src:               SourceList,
    pub script_src:                SourceList,
    pub style_src:                 SourceList,
    pub img_src:                   SourceList,
    pub connect_src:               SourceList,
    pub font_src:                  SourceList,
    pub object_src:                SourceList,
    pub media_src:                 SourceList,
    pub frame_src:                 SourceList,
    pub sandbox:                   Sandbox,
    pub report_uri:                &'static str,
    pub child_src:                 SourceList,
    pub form_action:               &'static str,
    pub frame_ancestors:           &'static str,
    pub plugin_types:              &'static str,
    pub base_uri:                  &'static str,
    pub report_to:                 &'static str,
    pub worker_src:                SourceList,
    pub manifest_src:              SourceList,
    pub prefetch_src:              SourceList,
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
/// ```
/// use ohkami::prelude::*;
/// use ohkami::fang::{Helmet, Sandbox::{allow_forms, allow_same_origin}};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Helmet {
///             sandbox: allow_forms | allow_same_origin,
///             ..Default::default()
///         },
///         "/hello".GET(|| async {"Hello, helmet!"})
///     )).howl("localhost:3000").await
/// }
/// ```
#[derive(Clone)]
pub struct Sandbox(u16);
const _: () = {
    #[allow(non_upper_case_globals)]
    impl Sandbox {
        pub const allow_forms:                    Self = Self(0b0000000001u16);
        pub const allow_same_origin:              Self = Self(0b0000000010u16);
        pub const allow_scripts:                  Self = Self(0b0000000100u16);
        pub const allow_popups:                   Self = Self(0b0000001000u16);
        pub const allow_modals:                   Self = Self(0b0000010000u16);
        pub const allow_orientation_lock:         Self = Self(0b0000100000u16);
        pub const allow_pointer_lock:             Self = Self(0b0001000000u16);
        pub const allow_presentation:             Self = Self(0b0010000000u16);
        pub const allow_popups_to_escape_sandbox: Self = Self(0b0100000000u16);
        pub const allow_top_navigation:           Self = Self(0b1000000000u16);
    }

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
        pub(self) const fn is_empty(&self) -> bool {
            self.0 == 0b0000000000u16
        }

        pub(self) fn build(&self) -> String {
            let mut result = String::new();
            if self.0 & Self::allow_forms.0 != 0                    {result.push_str(" allow-forms");}
            if self.0 & Self::allow_same_origin.0 != 0              {result.push_str(" allow-same-origin");}
            if self.0 & Self::allow_scripts.0 != 0                  {result.push_str(" allow-scripts");}
            if self.0 & Self::allow_popups.0 != 0                   {result.push_str(" allow-popups");}
            if self.0 & Self::allow_modals.0 != 0                   {result.push_str(" allow-modals");}
            if self.0 & Self::allow_orientation_lock.0 != 0         {result.push_str(" allow-orientation-lock");}
            if self.0 & Self::allow_pointer_lock.0 != 0             {result.push_str(" allow-pointer-lock");}
            if self.0 & Self::allow_presentation.0 != 0             {result.push_str(" allow-presentation");}
            if self.0 & Self::allow_popups_to_escape_sandbox.0 != 0 {result.push_str(" allow-popups-to-escape-sandbox");}
            if self.0 & Self::allow_top_navigation.0 != 0           {result.push_str(" allow-top-navigation");}
            result
        }
    }
};

/// ## Example
/// 
/// ```
/// use ohkami::prelude::*;
/// use ohkami::fang::{Helmet, SouceList::{self_origin, data}};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Helmet {
///             script_src: self_origin | data,
///             ..Default::default()
///         },
///         "/hello".GET(|| async {"Hello, helmet!"})
///     )).howl("localhost:3000").await
/// }
/// ```
#[derive(Clone, Default)]
pub struct SourceList {
    this: std::borrow::Cow<'static, str>,
    list: Vec<std::borrow::Cow<'static, str>>,
}
const _: () = {
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
        pub(self) const fn build_const(&self) -> std::borrow::Cow<'static, str> {
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
        pub(self) fn build_hash(&self) -> std::borrow::Cow<'static, str> {
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
    #[allow(non_upper_case_globals)]
    impl SourceList {
        pub const any:            Self = this!(const Source::any);
        pub const data:           Self = this!(const Source::data);
        pub const https:          Self = this!(const Source::https);
        pub const none:           Self = this!(const Source::none);
        pub const self_origin:    Self = this!(const Source::self_origin);
        pub const strict_dynamic: Self = this!(const Source::strict_dynamic);
        pub const unsafe_inline:  Self = this!(const Source::unsafe_inline);
        pub const unsafe_eval:    Self = this!(const Source::unsafe_eval);
        pub const unsafe_hashes:  Self = this!(const Source::unsafe_hashes);
        pub fn domain(domain: &'static str) -> Self {this!(const Source::domain(domain))}
        pub fn sha256(sha256: String) -> Self {this!(hash Source::sha256(sha256))}
        pub fn sha384(sha384: String) -> Self {this!(hash Source::sha384(sha384))}
        pub fn sha512(sha512: String) -> Self {this!(hash Source::sha512(sha512))}
        pub fn nonce (nonce:  String) -> Self {this!(hash Source::nonce(nonce))}
    }

    impl std::ops::BitOr for SourceList {
        type Output = Self;

        fn bitor(mut self, rhs: Self) -> Self::Output {
            self.list.push(rhs.this);
            self.list.extend(rhs.list);
            self
        }
    }

    impl SourceList {
        pub(self) fn is_empty(&self) -> bool {
            self.this.is_empty()
        }

        pub(self) fn build(&self) -> String {
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
};

const _: () = {
    use crate::{Request, Response, Fang, FangProc};
    use std::sync::OnceLock;

    impl<I: FangProc> Fang<I> for Helmet {
        type Proc = HelmetProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            static SET_HEADERS: OnceLock<Box<dyn Fn(&mut Response) + Send + Sync>> = OnceLock::new();

            let set_headers = SET_HEADERS.get_or_init(|| {
                /* clone only once */
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
