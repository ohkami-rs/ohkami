#[inline(always)] pub(crate) fn now() -> String {
    let mut now = chrono::Utc::now().to_rfc2822(); // like `Wed, 21 Dec 2022 10:16:52 +0000`
    match now.len() {
        30 => now.replace_range(25.., "GMT"),
        31 => now.replace_range(26.., "GMT"),
         _ => unreachable!()
    }
    now
}
