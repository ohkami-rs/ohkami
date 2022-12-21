pub(crate) fn tcp_address(address_str: &'static str) -> String {
    if address_str.starts_with(":") {
        "0.0.0.0".to_owned() + address_str
    } else if address_str.starts_with("localhost") {
        address_str.replace("localhost", "127.0.0.1")
    } else {
        address_str.to_owned()
    }
}

pub(crate) fn valid_path(path_str: &str) -> (bool, u8) {
    if !path_str.starts_with('/') {return (false, 0)}
    if path_str.len() == 1 {return (true, 0)}
    
    let mut path_param_count: u8 = 0;

    for section in path_str[1..].split('/') {
        let (is_valid, is_param) = valid_section(section);
        if is_param {path_param_count += 1}
        if !is_valid {return (false, path_param_count)}
    }

    (true, path_param_count)
}
fn valid_section(section: &str) -> (bool, bool/* is param */) {
    let mut is_param = false;

    let section = if section.starts_with(':') {
        is_param = true;

        if section.len() < 2 {return (false, false)}
        &section[1..]
    } else {
        if section.len() < 1 {return (false, false)}
        section
    };
    for ch in section.chars() {
        match ch {
            'a'..='z' | '_' => (),
            _ => return (false, is_param),
        }
    }

    (true, is_param)
}


#[cfg(test)]
mod test {
    use super::valid_path;

    #[test]
    fn validate_ok_paths() {
        assert_eq!(valid_path("/"), (true, 0));
        assert_eq!(valid_path("/api"), (true, 0));
        assert_eq!(valid_path("/api/:id"), (true, 1));
        assert_eq!(valid_path("/:number"), (true, 1));
        assert_eq!(valid_path("/api/users/:id"), (true, 1));
        assert_eq!(valid_path("/sleepy/:time/:name"), (true, 2));
    }
    #[test]
    fn validate_bad_paths() {
        assert_eq!(valid_path("//").0, false);
        assert_eq!(valid_path("/api/").0, false);
        assert_eq!(valid_path("/:id/").0, false);
        assert_eq!(valid_path(":id").0, false);
        assert_eq!(valid_path("api/").0, false);
        assert_eq!(valid_path("/:").0, false);
    }
}
