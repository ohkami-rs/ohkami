use crate::{Context, Response, Route, layer3_fang_handler::{IntoHandler, Handler, Fang, IntoFang}, Request};
use super::super::trie::*;
use Pattern::*;


async fn h(c: Context) -> Response {c.NoContent()}
fn H() -> Handler {h.into_handler()}

async fn f1(c: &mut Context) {
    c.headers
        .ETag("etagetagetag");
}
fn F1() -> Fang {f1.into_fang()}

async fn f2(c: &mut Context) {
    c.headers
        .Server("ohkami");
}
fn F2() -> Fang {f2.into_fang()}

async fn f3(req: &Request) {
    let __method__ = req.method();
    let __path__   = req.path();
    println!("Request {{ method: {__method__}, path: {__path__} }}");
}
fn F3() -> Fang {f3.into_fang()}

fn root(handler: Option<fn()->Handler>, children: Vec<Node>) -> Node {
    Node { pattern: None, fangs: vec![], handler: handler.map(|ih| ih()), children }
}


#[test] fn test_register_handlers() {
    fn node(pattern: Pattern, handler: Option<fn()->Handler>, children: Vec<Node>) -> Node {
        Node { pattern: Some(pattern), fangs: vec![], handler: handler.map(|ih| ih()), children }
    }

    let built = TrieRouter::new()
        .register_handlers("/"                 .GET(h))
        .register_handlers("/abc"              .GET(h))
        .register_handlers("/abc/:def"         .GET(h))
        .register_handlers("/api/xyz"          .GET(h))
        .register_handlers("/api/xyz/pqr"      .GET(h))
        .register_handlers("/api/xyz/pqr/final".GET(h))
        .register_handlers("/api/xyz/zyx"      .GET(h));

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


#[test] fn test_apply_fang() {
    fn node(pattern: Pattern, fangs: Vec<fn()->Fang>, children: Vec<Node>) -> Node {
        Node {
            pattern: Some(pattern),
            handler: None,
            fangs: fangs.into_iter().map(|into_fang| into_fang()).collect(),
            children,
        }
    }

    let built = TrieRouter::new()
        .apply_fang(F1())
        .apply_fang(F2())
        .apply_fang(F3());

    let correct = TrieRouter {
        GET: root(None, vec![]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct)
}
