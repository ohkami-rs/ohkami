use crate::{Context, Response, Route, layer3_fang_handler::{IntoHandler, Handler}};
use super::super::trie::*;
use Pattern::*;

pub async fn h1(c: Context) -> Response {c.NoContent()}
pub async fn h2(c: Context) -> Response {c.NoContent()}
pub async fn h3(c: Context) -> Response {c.NoContent()}
pub async fn h4(c: Context) -> Response {c.NoContent()}
pub async fn h5(c: Context) -> Response {c.NoContent()}
pub async fn h6(c: Context) -> Response {c.NoContent()}
pub async fn h7(c: Context) -> Response {c.NoContent()}

#[test] fn test_register_handlers() {
    #[allow(non_snake_case)] let H = || h1.into_handler();
    fn root(handler: Option<fn()->Handler>, children: Vec<Node>) -> Node {
        Node { pattern: None, fangs: vec![], handler: handler.map(|ih| ih()), children }
    }
    fn node(pattern: Pattern, handler: Option<fn()->Handler>, children: Vec<Node>) -> Node {
        Node { pattern: Some(pattern), fangs: vec![], handler: handler.map(|ih| ih()), children }
    }

    let built = TrieRouter::new()
        .register_handlers("/"                 .GET(h1))
        .register_handlers("/abc"              .GET(h2))
        .register_handlers("/abc/:def"         .GET(h3))
        .register_handlers("/api/xyz"          .GET(h4))
        .register_handlers("/api/xyz/pqr"      .GET(h5))
        .register_handlers("/api/xyz/pqr/final".GET(h6))
        .register_handlers("/api/xyz/zyx"      .GET(h7));

    let correct = TrieRouter {
        GET: root(Some(H), vec![
            node(Static{route: b"/abc", range: 1..4}, Some(H), vec![
                node(Param, Some(H), vec![])
            ]),
            node(Static{route: b"/api/xyz", range: 1..4}, None, vec![
                node(Static{route: b"/api/xyz", range: 5..8}, Some(H), vec![
                    node(Static{route: b"/api/xyz/pqr", range: 9..12}, Some(H), vec![
                        node(Static{route: b"/api/xyz/pqr/final", range: 13..18}, Some(H), vec![])
                    ]),
                    node(Static{route: b"/api/xyz/zyx", range: 9..12}, Some(H), vec![])
                ])
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);
}
