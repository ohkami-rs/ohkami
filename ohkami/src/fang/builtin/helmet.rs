pub struct Helmet(Option<Box<HelmetFields>>);

struct HelmetFields {
    delete_x_powered_by: bool,
    content_security_policy: Option<ContentSecurityPolicy>,
}

impl Default for Helmet {
    fn default() -> Self {
        Self(Some(Box::new(HelmetFields {
            delete_x_powered_by: true,
            content_security_policy: None,
        })))
    }
}

impl Helmet {
    pub fn delete_XPoweredBy(mut self, yes: bool) -> Self {
        self.delete_XPoweredBy = yes;
        self
    }
    pub fn ContentSecurityPolicy(mut self, setter: impl FnOnce(field::ContentSecurityPolicy) -> field::ContentSecurityPolicy) -> Self {
        self.ContentSecurityPolicy = setter(ContentSecurityPolicy(String::new()));
        self
    }
    pub fn ContentSecurityPolicyReportOnly(mut self, setter: impl FnOnce(field::ContentSecurityPolicy) -> field::ContentSecurityPolicy) -> Self {
        self.ContentSecurityPolicyReportOnly = setter(ContentSecurityPolicyReportOnly(String::new()));
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
