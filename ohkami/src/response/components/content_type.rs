#[allow(non_camel_case_types)]
pub(crate) enum ContentType {
    text_plain,
    text_html,
    application_json,
} impl ContentType {
    #[inline] pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::application_json => "application/json",
            Self::text_html => "text/html",
            Self::text_plain => "text/plain",
        }
    }
}
