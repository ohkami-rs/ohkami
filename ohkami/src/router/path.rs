use std::str::Split;

pub(crate) type Route = Path<'static>;
pub(crate) struct Path<'p>(
    Vec<Split<'p, char>>
); impl<'p> Path<'p> {
    pub(crate) fn handler(route: Vec<&'static str>) -> std::result::Result<Self, String> {
        is_valid_handler_route(route)?;
        Ok(Self(
            todo!()
        ))
    }
    pub(crate) fn fang(route: &'static str) -> std::result::Result<Self, String> {
        is_valid_fang_route(route)?;
        Ok(Self(vec![{
            let mut route = route
                .trim_end_matches('/')
                .split('/');
            {route.next();}
            route
        }]))
    }
}


///    / | (/:?[a-z, A-Z, _ ]+)+
#[inline] fn is_valid_handler_route(route: &str) -> std::result::Result<u8/* param count */, String> {
    if !route.starts_with('/') {return (false, 0)}
    if route.len() == 1 /* e.g. path.str == "/" */ {return (true, 0)}
    
    let mut path_param_count: u8 = 0;

    for section in route[1..].split('/') {
        let (is_valid, is_param) = is_valid_handler_route_section(section);
        if is_param {path_param_count += 1}
        if !is_valid {return (false, path_param_count)}
    }

    (true, path_param_count)
}
#[inline] fn is_valid_handler_route_section(section: &str) -> std::result::Result<bool/* is param */, String> {
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


///   / ([a-z, A-Z, _ ]+/)* \*?
#[inline] fn is_valid_fang_route(route: &'static str) -> std::result::Result<(), String> {
    if route == "*" {return Ok(())}
    
    if !route.starts_with('/') {return Err(format!("fang route `{route}` doesn't starts with `/`"))}
    if route.len() == 1 /* e.g. route == "/" */ {return Ok(())}

    let route = if route.ends_with("/*") {
        if route == "/*" {return Ok(())}
        &route[..route.len()-2]
    } else {route};

    for section in route[1..].split('/') {
        is_valid_fang_route_section(section)
            .map_err(|err_msg| format!("{err_msg}: in fang route `{route}`"))?
    }

    Ok(())
}
#[inline] fn is_valid_fang_route_section(section: &str) -> std::result::Result<(), String> {
    if section.len() < 1 {return Err(format!("empty route section"))}

    for ch in section.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => (),
            _ => return Err(format!("section `{section}` contains invalid charactor: `{ch}`"))
        }
    }

    Ok(())
}