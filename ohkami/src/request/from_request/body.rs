use serde::Deserialize;

pub struct Body<B: for <'b> Deserialize<'b>>(B);
