#![cfg(test)]
#![allow(unused)]

use super::Router;
use crate::test_system::Method::*;

#[test]
fn search_one_1() {
    let mock_handler = 100;

    let mut router = Router::new();
    router.register(GET, "/", mock_handler);

    assert_eq!(
        router.search(GET, "/"),
        Some(&mock_handler)
    )
}
#[test]
fn search_one_2() {
    let mock_handler = 100;

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler);

    assert_eq!(
        router.search(GET, "/api"),
        Some(&mock_handler)
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
        Some(&mock_handler_1)
    );
    assert_eq!(
        router.search(POST, "/api"),
        Some(&mock_handler_2)
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
        Some(&mock_handler_1)
    );
    assert_eq!(
        router.search(GET, "/api_v2"),
        Some(&mock_handler_2)
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
        Some(&mock_handler_1)
    );
    assert_eq!(
        router.search(GET, "/api/users"),
        Some(&mock_handler_2)
    );
}

#[test]
fn search_with_param() {
    let (mock_handler_1, mock_handler_2) = (100, 200);

    let mut router = Router::new();
    router.register(GET, "/api", mock_handler_1);
    router.register(GET, "/api/:id", mock_handler_2);

    assert_eq!(
        router.search(GET, "/api"),
        Some(&mock_handler_1)
    );
    assert_eq!(
        router.search(GET, "/api/2"),
        Some(&mock_handler_2)
    );
}