use std::borrow::Cow;
use crate::{Context, Response, Route, layer3_fang_handler::{IntoHandler, Handler, Fang, IntoFang, FrontFang}, Request, Ohkami};
use super::super::{trie, radix};


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
    const N_CHILDREN: usize,
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


#[test] fn into_raidx() {
    
}

