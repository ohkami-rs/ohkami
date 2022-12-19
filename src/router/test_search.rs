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