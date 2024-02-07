use std::borrow::Cow;

pub struct FxMap(rustc_hash::FxHashMap<Cow<'static, str>, Cow<'static, str>>);
