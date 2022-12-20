pub(crate) fn tcp_address(address_str: &'static str) -> String {
    if address_str.starts_with(":") {
        "0.0.0.0".to_owned() + address_str
    } else if address_str.starts_with("localhost") {
        address_str.replace("localhost", "127.0.0.1")
    } else {
        address_str.to_owned()
    }
}

pub(crate) fn is_valid_path(path_str: &str) -> bool {
    if !path_str.starts_with('/') {return false}
    if path_str.len() == 1 {return true}
    
    for section in path_str[1..].split('/') {
        if !is_valid_section(section) {
            return false
        }
    }

    true
}
fn is_valid_section(section: &str) -> bool {
    let section = if section.starts_with(':') {
        if section.len() == 1 {return false}
        &section[1..]
    } else {
        if section.len() == 0 {return false}
        section
    };
    for ch in section.chars() {
        match ch {
            'a'..='z' | '_' => (),
            _ => return false,
        }
    }
    true
}


#[cfg(test)]
mod test {
    use super::is_valid_path;

    #[test]
    fn validate_ok_paths() {
        assert!(is_valid_path("/"));
        assert!(is_valid_path("/api"));
        assert!(is_valid_path("/api/:id"));
        assert!(is_valid_path("/:number"));
        assert!(is_valid_path("/api/users/:id"));
        assert!(is_valid_path("/sleepy/:time/:name"));
    }
    #[test]
    fn validate_bad_paths() {
        assert_eq!(is_valid_path("//"), false);
        assert_eq!(is_valid_path("/api/"), false);
        assert_eq!(is_valid_path("/:id/"), false);
        assert_eq!(is_valid_path(":id"), false);
        assert_eq!(is_valid_path("api/"), false);
        assert_eq!(is_valid_path("/:"), false);
    }
}
