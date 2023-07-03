use std::borrow::Cow;

use crate::{Context, Response, Route, layer3_fang_handler::{IntoHandler, Handler, Fang, IntoFang}, Request, Ohkami};
use super::super::trie::*;
use Pattern::*;


macro_rules! assert_eq {
    ($left:ident, $right:ident) => {
        if $left != $right {
            panic!("\n\
                \n\
                ===== {}:{}:{} =====\n\
                \n\
                [left]\n\
                {:#?}\n\
                \n\
                [right]\n\
                {:#?}\n\
            ", file!(), line!(), column!(), $left, $right)
        }
    };
}

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

fn node_static(pattern: &'static str, handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<Node>) -> Node {
    Node {
        pattern: Some(Pattern::Static(Cow::Borrowed(pattern.as_bytes()))),
        handler: handler.map(|ih| ih()),
        fangs: fangs.into_iter().map(|into_fang| into_fang()).collect(),
        children,
    }
}
fn node_param(handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<Node>) -> Node {
    Node {
        pattern: Some(Pattern::Param),
        handler: handler.map(|ih| ih()),
        fangs: fangs.into_iter().map(|into_fang| into_fang()).collect(),
        children,
    }
}
fn root(handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<Node>) -> Node {
    Node {
        pattern: None,
        fangs:   fangs.into_iter().map(|i_f| i_f()).collect(),
        handler: handler.map(|ih| ih()),
        children
    }
}


#[test] fn test_register_handlers() {
    let built = TrieRouter::new()
        .register_handlers("/"                 .GET(h))
        .register_handlers("/abc"              .GET(h))
        .register_handlers("/abc/:def"         .GET(h))
        .register_handlers("/api/xyz"          .GET(h))
        .register_handlers("/api/xyz/pqr"      .GET(h))
        .register_handlers("/api/xyz/pqr/final".GET(h))
        .register_handlers("/api/xyz/zyx"      .GET(h));

    let correct = TrieRouter {
        GET: root(Some(H), vec![], vec![
            node_static("abc", Some(H), vec![], vec![
                node_param(Some(H), vec![], vec![])
            ]),
            node_static("api", None, vec![], vec![
                node_static("xyz", Some(H), vec![], vec![
                    node_static("pqr", Some(H), vec![], vec![
                        node_static("final", Some(H), vec![], vec![])
                    ]),
                    node_static("zyx", Some(H), vec![], vec![])
                ])
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);
}


#[test] fn test_apply_fang() {
    /*===== 1 =====*/

    let built = TrieRouter::new()
        .apply_fang(F1())
        .apply_fang(F2())
        .apply_fang(F3());

    let correct = TrieRouter::new();

    // F1, F2, F3 are not registered to any Node
    // because no Node in `built` has handler
    assert_eq!(built, correct);


    /*===== 2 =====*/
    
    let built = TrieRouter::new()
        .register_handlers("/"         .GET(h))
        .register_handlers("/api/hello".GET(h))
        .apply_fang(F1())
        .apply_fang(F2());

    let correct = TrieRouter {
        GET: root(Some(H), vec![F1, F2], vec![
            node_static("api", None, vec![], vec![
                node_static("hello", Some(H), vec![F1, F2], vec![])
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);


    /*===== 3 =====*/

    let built = TrieRouter::new()
        .register_handlers("/"          .GET(h))
        .register_handlers("/api/hello" .GET(h))
        .register_handlers("/api/health".GET(h))
        .apply_fang(F1())
        .apply_fang(F2());

    let correct = TrieRouter {
        GET: root(Some(H), vec![F1, F2], vec![
            node_static("api", None, vec![], vec![
                node_static("hello", Some(H), vec![F1, F2], vec![]),
                node_static("health", Some(H), vec![F1, F2], vec![])
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);
}


#[test] fn merge_node_without_fangs() {
    /*===== 1 =====*/
    let built = TrieRouter::new()
        .register_handlers("/hc" .GET(h))
        .merge_another(    "/api".by(
            Ohkami::new()(
                "/users".GET(h),
                "/tasks".GET(h),
            )
        ));
    let correct = TrieRouter {
        GET: root(None, vec![], vec![
            node_static("hc", Some(H), vec![], vec![]),
            node_static("api", None, vec![], vec![
                node_static("users", Some(H), vec![], vec![]),
                node_static("tasks", Some(H), vec![], vec![]),
            ])
        ]),
        ..TrieRouter::new()
    };
    assert_eq!(built, correct);


    /*===== 2 =====*/
    let users_ohkami = Ohkami::new()(
        "/".
            GET(h),
        "/:id".
            GET(h),
    );

    let tasks_ohkami = Ohkami::new()(
        "/:id".
            GET(h),
    );

    let api_ohkami = Ohkami::new()(
        "/users".by(users_ohkami),
        "/tasks".by(tasks_ohkami),
    );

    let built = TrieRouter::new()
        .register_handlers("/hc" .GET(h))
        .merge_another(    "/api".by(api_ohkami));

    let correct = TrieRouter {
        GET: root(None, vec![], vec![
            node_static("hc", Some(H), vec![], vec![]),
            node_static("api", None, vec![], vec![
                node_static("users", Some(H), vec![], vec![
                    node_param(Some(H), vec![], vec![])
                ]),
                node_static("tasks", None, vec![], vec![
                    node_param(Some(H), vec![], vec![])
                ]),
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);
}


#[test] fn merge_node_with_fangs() {
    /*===== 1 =====*/
    let built = TrieRouter::new()
        .register_handlers("/hc" .GET(h))
        .merge_another(    "/api".by(
            Ohkami::with((f1, f2))(
                "/users".GET(h),
                "/tasks".GET(h),
            )
        ))
        .apply_fang(F3());
    let correct = TrieRouter {
        GET: root(None, vec![], vec![
            node_static("hc", Some(H), vec![F3], vec![]),
            node_static("api", None, vec![], vec![
                node_static("users", Some(H), vec![F1, F2, F3], vec![]),
                node_static("tasks", Some(H), vec![F1, F2, F3], vec![]),
            ])
        ]),
        ..TrieRouter::new()
    };
    assert_eq!(built, correct);


    /*===== 2 =====*/
    let users_ohkami = Ohkami::with((f1,))(
        "/".
            GET(h),
        "/:id".
            GET(h),
    );

    let tasks_ohkami = Ohkami::new()(
        "/:id".
            GET(h),
    );

    let api_ohkami = Ohkami::with((f2,))(
        "/users".by(users_ohkami),
        "/tasks".by(tasks_ohkami),
    );

    let built = TrieRouter::new()
        .register_handlers("/hc" .GET(h))
        .merge_another(    "/api".by(api_ohkami));

    let correct = TrieRouter {
        GET: root(None, vec![], vec![
            node_static("hc", Some(H), vec![], vec![]),
            node_static("api", None, vec![], vec![
                node_static("users", Some(H), vec![F1, F2], vec![
                    node_param(Some(H), vec![F1, F2], vec![])
                ]),
                node_static("tasks", None, vec![], vec![
                    node_param(Some(H), vec![F2], vec![])
                ]),
            ])
        ]),
        ..TrieRouter::new()
    };

    assert_eq!(built, correct);
}
