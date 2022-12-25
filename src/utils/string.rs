pub(crate) fn unescaped(s: String) -> String {
    let mut unescaped = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {chars.next();},
            '\\' => unescaped.push(chars.next().unwrap()),
            _ => unescaped.push(ch),
        }
    }
    unescaped
}

#[cfg(test)]
mod test {
    use super::unescaped;

    #[test] // ???
    fn test_unescaped() {
        let case = String::from("\"{\\\"username\\\": \\\"Taro\\\"}\"");
        let expected = String::from("{\"username\": \"Taro\"}");
        assert_ne!(unescaped(case), expected)
    }
}
