/* to run this test, go to ../router.rs and
   - commentout `server::Handler` in `use::crate::{}`
   - uncommentout `pub(self) type Handler = usize;`
   - uncomment out `#[derive(Debug, PartialEq)]` in ./node.rs and ../router.rs

#![cfg(test)]
#![allow(unused)]

use super::Router;
use crate::{test_system::Method::*, utils::map::StrMap};

#[test]
fn search_one_1() {
    let mock_handler = 100;

    let mut router = Router::new();
    router.register(GET, "/", mock_handler);

    assert_eq!(
        router.search(GET, "/"),
        Ok((&mock_handler, StrMap::new()))
    )
}
#[test]
fn search_one_2() {
    let mock_handler = 100;

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((&mock_handler, StrMap::new()))
    )
}

#[test]
fn search_two_pararel_1() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(POST, "/api", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((&mock_handler_1, StrMap::new()))
    );
    assert_eq!(
        router.search(POST, "/api"),
        Ok((&mock_handler_2, StrMap::new()))
    );
}
#[test]
fn search_two_pararel_2() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(GET, "/api_v2", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((&mock_handler_1, StrMap::new()))
    );
    assert_eq!(
        router.search(GET, "/api_v2"),
        Ok((&mock_handler_2, StrMap::new()))
    );
}

#[test]
fn search_two_nested() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(GET, "/api/users", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((&mock_handler_1, StrMap::new()))
    );
    assert_eq!(
        router.search(GET, "/api/users"),
        Ok((&mock_handler_2, StrMap::new()))
    );
}

#[test]
fn search_with_param_1() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(GET, "/api/:id", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((
            &mock_handler_1,
            StrMap::new()
        ))
    );
    assert_eq!(
        router.search(GET, "/api/2"),
        Ok((
            &mock_handler_2,
            StrMap {
                count: 1,
                map: [Some(("id", "2")), None, None, None]
            }
        ))
    );
}
#[test]
fn search_with_param_2() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(GET, "/api/:version/users/:id", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Ok((
            &mock_handler_1,
            StrMap::new()
        ))
    );
    assert_eq!(
        router.search(GET, "/api/1/users/3"),
        Ok((
            &mock_handler_2,
            StrMap {
                count: 2,
                map: [Some(("version", "1")), Some(("id", "3")), None, None]
            }
        ))
    );
}

*/
