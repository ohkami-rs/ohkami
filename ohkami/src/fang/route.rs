use std::{collections::VecDeque, ops::Range};


pub trait FangRoute {
    fn bite<const N: usize>(self, )
}

pub(super) struct FangsRoute(
    VecDeque<FangRoutePattern>
);
enum FangRoutePattern {
    Section {range: Range<usize>, route_str: &'static str},
    Param,
    AnyAfter,
}
