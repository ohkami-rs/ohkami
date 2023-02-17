use std::{pin::Pin, future::Future};
use crate::{router::route::Route, components::headers::RequestHeaders, response::ResponseWriter};

pub struct Fangs(Vec<(Route, Fang)>);
struct Fang(Box<dyn
    Fn(&RequestHeaders, &mut ResponseWriter) -> Pin<Box<dyn Future<Output=()> + Send>>
+ Send + Sync>);
