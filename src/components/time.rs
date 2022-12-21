use chrono::Utc;

pub(crate) fn now_fmt() -> String {
    let mut now_str = Utc::now().to_rfc2822(); // like `Wed, 21 Dec 2022 10:16:52 +0000`
    match now_str.len() {
        30 => now_str.replace_range(25.., "GMT"),
        31 => now_str.replace_range(26.., "GMT"),
         _ => unreachable!()
    }
    now_str
}