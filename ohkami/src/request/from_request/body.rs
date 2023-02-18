use crate::components::json::JSON;

pub struct Body<B: for <'j> JSON<'j>>(B);
