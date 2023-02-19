use std::str::Split;
use crate::error::Error;

pub(crate) struct HandlerRoute {
    param_count: u8,
    route:       Split<'static, char>,
}
pub(crate) struct FangRoute(
    Split<'static, char>
);

pub trait Route {
    fn into_handler_route(self) -> crate::Result<HandlerRoute>;
    fn into_fang_route(self) -> crate::Result<FangRoute>;
}
impl Route for &'static str {
    #[inline] fn into_handler_route(self) -> crate::Result<HandlerRoute> {
        let param_count = is_valid_handler_route(&self)?;
        
        Ok(HandlerRoute {
            param_count,
            route: {
                let mut split =  self.trim_end_matches('/').split('/');
                split.next();
                split
            }
        })
    }
    #[inline] fn into_fang_route(self) -> crate::Result<FangRoute> {
        is_valid_fang_route(&self)?;
        Ok(FangRoute({
            let mut split = self.trim_end_matches('/').split('/');
            split.next();
            split
        }))
    }
}

///    / | (/:?[a-z, A-Z, _ ]+)+
#[inline] fn is_valid_handler_route(route: &str) -> crate::Result<u8/* param count */> {
    if !route.starts_with('/') {return Err(Error::Validation(format!("route `{route}` doesn't start with `/`")))}
    if route.len() == 1 /* e.g. path.str == "/" */ {return Ok(0)}
    
    let mut path_param_count: u8 = 0;
    for section in route[1..].split('/') {
        if is_valid_handler_route_section(section)? {
            path_param_count += 1
        }
    }
    Ok(path_param_count)
}
#[inline] fn is_valid_handler_route_section(section: &str) -> crate::Result<bool/* is param */> {
    let mut is_param = false;

    let section = if section.starts_with(':') {
        is_param = true;
        if section.len() < 2 {return Err(Error::Validation(format!("`:` is not valid route section")))}
        &section[1..]
    } else {
        if section.len() < 1 {return Err(Error::Validation(format!("empty route section")))}
        section
    };
    for ch in section.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => (),
            _ => return Err(Error::Validation(format!("section `{section}` contains invalid charactor: `{ch}`. route section can contains: 'a'..='z' | 'A'..='Z' | '_'"))),
        }
    }

    Ok(is_param)
}


/// \*  |  / ([a-z, A-Z, _ ]+/)* \*?
#[inline] fn is_valid_fang_route(route: &'static str) -> crate::Result<()> {
    if route == "*" {return Ok(())}
    
    if !route.starts_with('/') {return Err(Error::Validation(format!("fang route `{route}` doesn't starts with `/`")))}
    if route.len() == 1 /* e.g. route == "/" */ {return Ok(())}

    let route = if route.ends_with("/*") {
        if route == "/*" {return Ok(())}
        &route[..route.len()-2]
    } else {route};

    for section in route[1..].split('/') {
        is_valid_fang_route_section(section)
            .map_err(|err_msg| Error::Validation(format!("{err_msg}: in fang route `{route}`")))?
    }

    Ok(())
}
#[inline] fn is_valid_fang_route_section(section: &str) -> crate::Result<()> {
    if section.len() < 1 {return Err(Error::Validation(format!("empty route section")))}

    for ch in section.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '_' => (),
            _ => return Err(Error::Validation(format!("section `{section}` contains invalid charactor: `{ch}`")))
        }
    }

    Ok(())
}