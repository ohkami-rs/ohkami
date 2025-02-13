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
