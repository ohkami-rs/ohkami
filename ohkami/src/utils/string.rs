pub(crate) fn unescaped(s: String) -> String {
    let mut unescaped = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"'  => (),
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
        let case = String::from("\"");
        let expected = String::from("");
        assert_eq!(unescaped(case), expected);
    
        let case = String::from("\"{\\\"username\\\": \\\"Taro\\\"}\"");
        let expected = String::from("{\"username\": \"Taro\"}");
        assert_eq!(unescaped(case), expected);
    }
}
