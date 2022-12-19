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
    
    let mut path_parts = path_str[1..].split('/');

    let Some(tail) = path_parts.next_back() else {return false};
    if tail.starts_with(':') {
        if !(tail.len() > 1) || !is_valid_path_part(&tail[1..]) {return false}
    } else {
        if !is_valid_path_part(tail) {return false}
    }
    
    while let Some(part) = path_parts.next_back() {
        if !is_valid_path_part(part) {return false}
    }

    true
}
fn is_valid_path_part(path_part_str: &str) -> bool {
    if !(path_part_str.len() > 0) {return false}
    for ch in path_part_str.chars() {
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

#[cfg(test)]
mod pre_test {
    use super::is_valid_path_part;

    #[test]
    fn how_fn_is_valid_path_part_works() {
        assert!(is_valid_path_part("api"));
        assert!(is_valid_path_part("_api"));

        assert_eq!(is_valid_path_part(":api"), false);
        assert_eq!(is_valid_path_part("/api"), false);
    }
    #[test]
    fn logic_in_fn_is_valid_path_part_has_no_problem() {
        assert!("a".len() > 0);
        assert!((|| {
            if !("a".len() > 0) {return false}
            true
        })());
        assert!((|| {
            if !("a".len() > 0) {return false}
            for ch in "a".chars() {
                match ch {
                    'a'..='z' | '_' => (),
                    _ => return false,
                }
            }
            true
        })());
        assert!(is_valid_path_part("a"));
    }

    #[test]
    fn how_fn_split_works() {
        let mut case = "/".split('/');
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), None);

        let mut case = "/api".split('/');
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), Some("api"));
        assert_eq!(case.next(), None);

        let mut case = "/api/users/:id".split('/');
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), Some("api"));
        assert_eq!(case.next(), Some("users"));
        assert_eq!(case.next(), Some(":id"));
    }
    #[test]
    fn how_fn_next_back_works() {
        let mut case = "/api"[1..].split('/');
        assert_eq!(case.next_back(), Some("api"));
        assert_eq!(case.next_back(), None);

        let mut case = "/api/users/:id"[1..].split('/');
        assert_eq!(case.next_back(), Some(":id"));
        assert_eq!(case.next_back(), Some("users"));
        assert_eq!(case.next_back(), Some("api"));
        assert_eq!(case.next_back(), None);
    }
    #[test]
    fn how_fn_len_works() {
        assert_eq!("api".len(), 3);
        assert_eq!(":api".len(), 4);
        assert_eq!("".len(), 0);
    }
    #[test]
    fn how_iterating_match_on_chars_works() {
        let result = |path_str: &str| {
            for ch in path_str.chars() {
                match ch {
                    'a'..='z' | '_' => (),
                    _ => return false,
                }
            }
            true
        };

        assert!(result("api"));
        assert!(result("_api"));

        assert_eq!(result("/api"), false);
        assert_eq!(result(":api"), false);
    }
}