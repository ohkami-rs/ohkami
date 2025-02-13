pub struct Helmet(Box<HelmetFields>);

/// based on <https://hono.dev/docs/middleware/builtin/secure-headers>,
/// with removing non-standard or deprecated headers
#[derive(Clone)]
struct HelmetFields {
    pub ContentSecurityPolicy:           Option<CSP>,
    pub ContentSecurityPolicyReportOnly: Option<CSP>,
    pub CrossOriginEmbedderPolicy:       Option<&'static str>,
    pub CrossOriginResourcePolicy:       Option<&'static str>,
    pub ReferrerPolicy:                  Option<&'static str>,
    pub StrictTransportSecurity:         Option<&'static str>,
    pub XContentTypeOptions:             Option<&'static str>,
    pub XFrameOptions:                   Option<&'static str>,
}

/// based on <https://content-security-policy.com>
#[derive(Clone)]
pub struct CSP {
    pub default_src:               Option<&'static str>,
    pub script_src:                Option<&'static str>,
    pub style_src:                 Option<&'static str>,
    pub img_src:                   Option<&'static str>,
    pub connect_src:               Option<&'static str>,
    pub font_src:                  Option<&'static str>,
    pub object_src:                Option<&'static str>,
    pub media_src:                 Option<&'static str>,
    pub frame_src:                 Option<&'static str>,
    pub sandbox:                   Option<Sandbox>,
    pub report_uri:                Option<&'static str>,
    pub child_src:                 Option<&'static str>,
    pub form_action:               Option<&'static str>,
    pub frame_ancestors:           Option<&'static str>,
    pub plugin_types:              Option<&'static str>,
    pub base_uri:                  Option<&'static str>,
    pub report_to:                 Option<&'static str>,
    pub worker_src:                Option<&'static str>,
    pub manifest_src:              Option<&'static str>,
    pub prefetch_src:              Option<&'static str>,
    pub navifate_to:               Option<&'static str>,
    pub require_trusted_types_for: Option<&'static str>,
    pub trusted_types:             Option<&'static str>,
    pub upgrade_insecure_requests: Option<&'static str>,
    pub block_all_mixed_content:   Option<&'static str>,
}

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
        pub const allow_forms                    = Self(0b0000000001u16),
        pub const allow_same_origin              = Self(0b0000000010u16),
        pub const allow_scripts                  = Self(0b0000000100u16),
        pub const allow_popups                   = Self(0b0000001000u16),
        pub const allow_modals                   = Self(0b0000010000u16),
        pub const allow_orientation_lock         = Self(0b0000100000u16),
        pub const allow_pointer_lock             = Self(0b0001000000u16),
        pub const allow_presentation             = Self(0b0010000000u16),
        pub const allow_popups_to_escape_sandbox = Self(0b0100000000u16),
        pub const allow_top_navigation           = Self(0b1000000000u16),
    }

    impl std::ops::BitOr for Sandbox {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self(self.0 | rhs.0)
        }
    }

    impl Sandbox {
        pub(self) fn build(&self) -> String {
            let mut result = String::from("sandbox");
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

pub struct SourceList {
    directive: u16,
    value: Option<Box<String>>,
}
const _: () = {
    #[allow(non_upper_case_globals)]
    impl SourceList {
        
    }
};

const _: () = {
    use crate::{Request, Response, Fang, FangProc};
    use std::sync::OnceLock;

    impl<I: FangProc> Fang for Helmet {
        type Proc = HelmetProc<I>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            static SET_HEADERS: OnceLock<Box<dyn Fn(&mut Request)>>;

            /* clone only once */
            let set_headers = SET_HEADERS.get_or_init({

                || {
                    
                }
            });

            HelmetProc { inner, set_headers }
        }
    }

    struct HelmetProc<I> {
        set_headers: Box<dyn Fn(&mut Request)>,
        inner: I,
    }

    impl<I: FangProc> FangProc<I> for HelmetProc<I> {

    }
};

impl Helmet {
    pub fn ContentSecurityPolicy(mut self, setter: impl FnOnce(field::ContentSecurityPolicy) -> field::ContentSecurityPolicy) -> Self {
        self.ContentSecurityPolicy = Some(setter(ContentSecurityPolicy(String::new())).0);
        self
    }
    pub fn ContentSecurityPolicyReportOnly(mut self, setter: impl FnOnce(field::ContentSecurityPolicy) -> field::ContentSecurityPolicy) -> Self {
        self.ContentSecurityPolicyReportOnly = Some(setter(ContentSecurityPolicyReportOnly(String::new())).0);
        self
    }
    pub fn CrossOriginEmbedderPolicy_require_corp(mut self) -> Self {
        self.CrossOriginEmbedderPolicy_require_corp = true;
        self
    }
}

mod field {
    struct ContentSecurityPolicy(String);
    impl ContentSecurityPolicy {

    }


}
