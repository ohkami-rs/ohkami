pub(crate) fn unescaped(s: String) -> String {
    let mut unescaped = String::with_capacity(s.len());
    let mut esc = false;
    for ch in s.chars() {
        if ch == '\\' {
            esc = true;
            continue
        } else if esc {
            esc = false;
            continue
        } else {
            unescaped.push(ch)
        }
    }
    unescaped
}

#[cfg(test)]
mod test {
    use super::unescaped;

    #[test]
    fn test_unescaped() {
        let case = String::from("\"{\\\"username\\\": \\\"Taro\\\"}\"");
        let expected = String::from("{\"username\": \"Taro\"}");
        assert_ne!(unescaped(case), expected)
    }
}
