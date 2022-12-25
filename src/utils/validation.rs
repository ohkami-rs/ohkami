pub(crate) fn tcp_address(address_str: &'static str) -> String {
    if address_str.starts_with(":") {
        "0.0.0.0".to_owned() + address_str
    } else if address_str.starts_with("localhost") {
        address_str.replace("localhost", "127.0.0.1")
    } else {
        address_str.to_owned()
    }
}

/*
    / | (/:?[a-z, A-Z, _ ]+)+
*/
pub(crate) fn valid_request_path(path_str: &str) -> (bool, u8) {
    if !path_str.starts_with('/') {return (false, 0)}
    if path_str.len() == 1 /* e.g. path.str == "/" */ {return (true, 0)}
    
    let mut path_param_count: u8 = 0;

    for section in path_str[1..].split('/') {
        let (is_valid, is_param) = valid_request_path_section(section);
        if is_param {path_param_count += 1}
        if !is_valid {return (false, path_param_count)}
    }

    (true, path_param_count)
}
fn valid_request_path_section(section: &str) -> (bool, bool/* is param */) {
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
            'a'..='z' | 'A'..='Z' | '_' => (),
            _ => return (false, is_param),
        }
    }

    (true, is_param)
}

/*
   / ([a-z, A-Z, _ ]+/)* \*?
*/
pub(crate) fn valid_middleware_route(route: &'static str) -> bool {
    if route == "*" {return true}
    
    if !route.starts_with('/') {return false}
    if route.len() == 1 /* e.g. route == "/" */ {return true}

    let route = if route.ends_with("/*") {
        if route == "/*" {return true}
        &route[..route.len()-2]
    } else {route};

    for section in route[1..].split('/') {
        if !is_valid_middleware_route_section(section) {
            return false
        }
    }

    true
}
fn is_valid_middleware_route_section(section: &str) -> bool {
    if section.len() < 1 {return false}

    for ch in section.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => (),
            _ => return false
        }
    }

    true
}


#[cfg(test)]
mod test {
    use super::{valid_request_path, valid_middleware_route};

    #[test]
    fn how_str_split_works() {
        let mut case = "/".split('/');
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), None);

        let mut case = "".split('/');
        assert_eq!(case.next(), Some(""));
        assert_eq!(case.next(), None);
    }

    #[test]
    fn validate_ok_request_paths() {
        assert_eq!(valid_request_path("/"), (true, 0));
        assert_eq!(valid_request_path("/api"), (true, 0));
        assert_eq!(valid_request_path("/api/:id"), (true, 1));
        assert_eq!(valid_request_path("/:number"), (true, 1));
        assert_eq!(valid_request_path("/api/users/:id"), (true, 1));
        assert_eq!(valid_request_path("/sleepy/:time/:name"), (true, 2));
    }
    #[test]
    fn validate_bad_request_paths() {
        assert_eq!(valid_request_path("").0, false);
        assert_eq!(valid_request_path("//").0, false);
        assert_eq!(valid_request_path("/api/").0, false);
        assert_eq!(valid_request_path("/:id/").0, false);
        assert_eq!(valid_request_path(":id").0, false);
        assert_eq!(valid_request_path("api/").0, false);
        assert_eq!(valid_request_path("/:").0, false);
    }

    #[test]
    fn validate_ok_middleware_routes() {
        assert!(valid_middleware_route("*"));
        assert!(valid_middleware_route("/"));
        assert!(valid_middleware_route("/*"));
        assert!(valid_middleware_route("/api/*"));
        assert!(valid_middleware_route("/api/users/*"));
    }
    #[test]
    fn validate_bad_middleware_routes() {
        assert!( ! valid_middleware_route("/*/api"));
        assert!( ! valid_middleware_route(""));
        assert!( ! valid_middleware_route("//*"));
        assert!( ! valid_middleware_route("*/"));
        assert!( ! valid_middleware_route("api/*"));
    }
}
