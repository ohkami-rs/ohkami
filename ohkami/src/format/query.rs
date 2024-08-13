use crate::FromRequest;
use serde::Deserialize;


pub struct Query<'q, S: Deserialize<'q>>(pub S);

//impl<'req> FromRequest<'req> for Query<>
