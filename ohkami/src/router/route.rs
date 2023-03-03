use std::{ops::Range, collections::VecDeque};
use super::Pattern;


pub trait Route {
    fn into_handle_route(self) -> HandleRoute;
    fn into_fang_route(self)   -> FangRoute;
}

pub struct HandleRoute(
    VecDeque<Pattern>
);

pub struct FangRoute(
    VecDeque<FangRoutePattern>
); pub(super) enum FangRoutePattern {
    Section {route_str: &'static str, range: Range<usize>},
    Param,
    AnyAfter,
}


impl Route for &'static str {
    fn into_handle_route(self) -> HandleRoute {
        
    }
    fn into_fang_route(self)   -> FangRoute {
        
    }
}


const _: (/* HandleRoute impls */) = {
    impl Iterator for HandleRoute {
        type Item = Pattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
const _: (/* FangRoute impls */) = {
    impl Iterator for FangRoute {
        type Item = FangRoutePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
