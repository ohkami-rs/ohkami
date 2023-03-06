use std::ops::Range;

pub(crate) enum TriePattern {
    Section { route_str: &'static str, range: Range<usize> },
    Param,
    Nil,
} impl TriePattern {
    pub(crate) fn parse(range: Range<usize>, route_str: &str) -> Self {
        let section = &route_str[range];
        if section.starts_with(':') {
            let param_name = &section[1..];
            if let Err(msg) = validate_section(param_name) {
                panic!("path parameter `{param_name}` in route `{route_str}` is invalid: \"{msg}\"");
            }
            Self::Param
        } else {
            if let Err(msg) = validate_section(section) {
                panic!("section `{section}` in route `{route_str}` is invalid: \"{msg}\"");
            }
            Self::Section { route_str, range }
        }
    }
}

pub(crate) fn validate_section(section: &str) -> Result<(), &'static str> {
    match section.len() {
        0 => Err("empty string"),
        1 => match section.chars().next().unwrap() {
            'a'..='z' | 'A'..='Z' => Ok(()),
            _ => Err("route section or path param's name must starts with 'a'..='z' | 'A'..='Z'"),
        },
        _ => {
            let chars = section.chars();
            match chars.next().unwrap() {
                'a'..='z' | 'A'..='Z' => (),
                _ => return Err("route section or path param's name must start with 'a'..='z' | 'A'..='Z'"),
            };
            for ch in chars {
                match ch {
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => (),
                    _ => return Err("route section or path param's name must consist of 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'"),
                }
            }
            Ok(())
        },
    }
}
