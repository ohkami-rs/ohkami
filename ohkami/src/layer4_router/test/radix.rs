use std::borrow::Cow;
use radix::RadixRouter;
use trie::TrieRouter;

use crate::{Context, Response, layer3_fang_handler::{IntoHandler, Handler, Fang, IntoFang, FrontFang}, Request};
use super::super::{trie, radix, radix::Pattern::*};


macro_rules! assert_eq {
    ($left:ident, $right:expr) => {
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

async fn f1(c: &mut Context) {c.headers.ETag("etagetagetag");}
fn F1() -> Fang {f1.into_fang()}

async fn f2(c: &mut Context) {c.headers.Server("ohkami");}
fn F2() -> Fang {f2.into_fang()}

async fn f3(req: &Request) {
    let __method__ = req.method();
    let __path__   = req.path();
    println!("Request {{ method: {__method__}, path: {__path__} }}");
}
fn F3() -> Fang {f3.into_fang()}


fn trie_static(pattern: &'static str, handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<trie::Node>) -> trie::Node {
    trie::Node {
        pattern: Some(trie::Pattern::Static(Cow::Borrowed(pattern.as_bytes()))),
        handler: handler.map(|ih| ih()),
        fangs: fangs.into_iter().map(|into_fang| into_fang()).collect(),
        children,
    }
}
fn trie_param(handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<trie::Node>) -> trie::Node {
    trie::Node {
        pattern: Some(trie::Pattern::Param),
        handler: handler.map(|ih| ih()),
        fangs: fangs.into_iter().map(|into_fang| into_fang()).collect(),
        children,
    }
}
fn trie_root(handler: Option<fn()->Handler>, fangs: Vec<fn()->Fang>, children: Vec<trie::Node>) -> trie::Node {
    trie::Node {
        pattern: None,
        fangs:   fangs.into_iter().map(|i_f| i_f()).collect(),
        handler: handler.map(|ih| ih()),
        children
    }
}


fn radix<
    const N_PATTERNS: usize,
    const N_FRONT:    usize,
>(
    patterns: [radix::Pattern; N_PATTERNS],
    handler:  Option<fn()->Handler>,
    front:    [FrontFang; N_FRONT],
    children: Vec<radix::Node>,
) -> radix::Node {
    radix::Node {
        patterns: patterns.to_vec().leak(),
        front:    front.to_vec().leak(),
        handler:  handler.map(|ih| ih()),
        children,
    }
}

fn emptyRadixRouter() -> radix::RadixRouter {
    radix::RadixRouter {
        GET:     radix([], None, [], vec![]),
        PUT:     radix([], None, [], vec![]),
        POST:    radix([], None, [], vec![]),
        HEAD:    radix([], None, [], vec![]),
        PATCH:   radix([], None, [], vec![]),
        DELETE:  radix([], None, [], vec![]),
        OPTIONS: radix([], None, [], vec![]),
    }
}


#[test] fn into_radix_without_fangs() {
    /*===== 1 =====*/

    let built = TrieRouter {
        GET: trie_root(Some(H), vec![], vec![]),
        ..TrieRouter::new()
    }.into_radix();

    let correct = RadixRouter {
        GET: radix([], Some(H), [], vec![]),
        ..emptyRadixRouter()
    };

    assert_eq!(built, correct);


    /*===== 2 =====*/

    let built = TrieRouter {
        GET: trie_root(None, vec![], vec![
            trie_static("hc",  Some(H), vec![], vec![]),
            trie_static("api", None,    vec![], vec![
                trie_static("hello", Some(H), vec![], vec![])
            ])
        ]),
        ..TrieRouter::new()
    }.into_radix();

    let correct = RadixRouter {
        GET: radix([], None, [], vec![
            radix([Static(b"hc")],        Some(H), [], vec![]),
            radix([Static(b"api/hello")], Some(H), [], vec![]),
        ]),
        ..emptyRadixRouter()
    };

    assert_eq!(built, correct);


    /*===== 3 =====*/

    let built = TrieRouter {
        GET: trie_root(None, vec![], vec![
            trie_static("hc",  Some(H), vec![], vec![]),
            trie_static("api", None,    vec![], vec![
                trie_static("hello", None, vec![], vec![
                    trie_static("v1", None, vec![], vec![
                        trie_static("with_repeat", None, vec![], vec![
                            trie_param(Some(H), vec![], vec![])
                        ])
                    ])
                ]),
                trie_static("users", Some(H), vec![], vec![
                    trie_param(Some(H), vec![], vec![])
                ]),
                trie_static("tasks", None, vec![], vec![
                    trie_param(Some(H), vec![], vec![])
                ])
            ])
        ]),
        ..TrieRouter::new()
    }.into_radix();

    let correct = RadixRouter {
        GET: radix([], None, [], vec![
            radix([Static(b"hc")], Some(H), [], vec![]),
            radix([Static(b"api")], None, [], vec![
                radix([Static(b"hello/v1/with_repeat"), Param], Some(H), [], vec![]),
                radix([Static(b"users")], Some(H), [], vec![
                    radix([Param], Some(H), [], vec![])
                ]),
                radix([Static(b"tasks"), Param], Some(H), [], vec![])
            ])
        ]),
        ..emptyRadixRouter()
    };

    assert_eq!(built, correct);
}


#[test] fn test_search() {
    macro_rules! assert_search {
        ($router:ident {
            $(
                $path:literal => $expected:expr
            )*
        }) => {
            $(
                let (found, expected) = ($router.GET.search($path.as_bytes()).map(|(node, _)| node), $expected);
                match found {
                    None => assert_eq!(found, (&expected).as_ref()),
                    Some(found_node) => {
                        if found_node.handler.is_some() {
                            assert_eq!(found, (&expected).as_ref())
                        } else {
                            ::std::assert_eq!(None, (&expected).as_ref());
                        }
                    }
                }
            )*
        };
    }


    let router = RadixRouter {
        GET: radix([], None, [], vec![
            radix([Static(b"hc")], Some(H), [], vec![]),
            radix([Static(b"api")], None, [], vec![
                radix([Static(b"hello/v1/with_repeat"), Param], Some(H), [], vec![]),
                radix([Static(b"users")], Some(H), [], vec![
                    radix([Param], Some(H), [], vec![])
                ]),
                radix([Static(b"tasks"), Param], Some(H), [], vec![])
            ])
        ]),
        ..emptyRadixRouter()
    };

    assert_search!(router {
        "/hc"  => Some(radix([Static(b"hc")], Some(H), [], vec![]))
        "/api" => None
        "/api/hello/v1/with_repeat"     => None
        "/api/hello/v1/with_repeat/100" => Some(radix([Static(b"hello/v1/with_repeat"), Param], Some(H), [], vec![]))
        "/api/users"    => Some(radix([Static(b"users")], Some(H), [], vec![radix([Param], Some(H), [], vec![])]))
        "/api/users/42" => Some(radix([Param], Some(H), [], vec![]))
        "/" => None
        "/api/tasks"         => None
        "/api/tasks/3141592" => Some(radix([Static(b"tasks"), Param], Some(H), [], vec![]))
    });
}
