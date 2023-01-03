use super::{HandleFunc, Handler, Param};

#[allow(non_snake_case)]
pub struct HandlerGroup {
    pub(crate) max_param_count: u8,
    pub(crate) GET:    Option<HandleFunc>,
    pub(crate) POST:   Option<HandleFunc>,
    pub(crate) PATCH:  Option<HandleFunc>,
    pub(crate) DELETE: Option<HandleFunc>,
} impl HandlerGroup {
    #[allow(non_snake_case)]
    pub fn GET<H: Handler<P>, P: Param>(mut self, handler: H) -> Self {
        let (handler, param_count) = handler.into_handlefunc();
        self.max_param_count = self.max_param_count.max(param_count);
        HandlerGroup {
            GET: Some(handler),
            ..self
        }
    }
    #[allow(non_snake_case)]
    pub fn POST<H: Handler<P>, P: Param>(mut self, handler: H) -> Self {
        let (handler, param_count) = handler.into_handlefunc();
        self.max_param_count = self.max_param_count.max(param_count);
        HandlerGroup {
            POST: Some(handler),
            ..self
        }
    }
    #[allow(non_snake_case)]
    pub fn PATCH<H: Handler<P>, P: Param>(mut self, handler: H) -> Self {
        let (handler, param_count) = handler.into_handlefunc();
        self.max_param_count = self.max_param_count.max(param_count);
        HandlerGroup {
            PATCH: Some(handler),
            ..self
        }
    }
    #[allow(non_snake_case)]
    pub fn DELETE<H: Handler<P>, P: Param>(mut self, handler: H) -> Self {
        let (handler, param_count) = handler.into_handlefunc();
        self.max_param_count = self.max_param_count.max(param_count);
        HandlerGroup {
            DELETE: Some(handler),
            ..self
        }
    }
}

#[allow(non_snake_case, unused)]
pub fn GET<H: Handler<P>, P: Param>(handler: H) -> HandlerGroup {
    let (handler, param_count) = handler.into_handlefunc();
    HandlerGroup {
        max_param_count: param_count,
        GET:    Some(handler),
        POST:   None,
        PATCH:  None,
        DELETE: None,
    }
}
#[allow(non_snake_case, unused)]
pub fn POST<H: Handler<P>, P: Param>(handler: H) -> HandlerGroup {
    let (handler, param_count) = handler.into_handlefunc();
    HandlerGroup {
        max_param_count: param_count,
        GET:    None,
        POST:   Some(handler),
        PATCH:  None,
        DELETE: None,
    }
}
#[allow(non_snake_case, unused)]
pub fn PATCH<H: Handler<P>, P: Param>(handler: H) -> HandlerGroup {
    let (handler, param_count) = handler.into_handlefunc();
    HandlerGroup {
        max_param_count: param_count,
        GET:    None,
        POST:   None,
        PATCH:  Some(handler),
        DELETE: None,
    }
}
#[allow(non_snake_case, unused)]
pub fn DELETE<H: Handler<P>, P: Param>(handler: H) -> HandlerGroup {
    let (handler, param_count) = handler.into_handlefunc();
    HandlerGroup {
        max_param_count: param_count,
        GET:    None,
        POST:   None,
        PATCH:  None,
        DELETE: Some(handler),
    }
}