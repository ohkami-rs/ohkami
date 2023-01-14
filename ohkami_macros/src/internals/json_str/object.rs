use std::{collections::BTreeMap, fmt::Debug};
use super::JsonStr;

pub(crate) struct Object(
    pub(crate) BTreeMap<String, JsonStr>
);

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = self.0.clone();

        write!(f, "{{{}}}", 'content: {
            let mut content = String::new();

            let Some((first_key, first_value)) = map.pop_first() else {
                break 'content content
            };
            content += &format!("{first_key}:{first_value:?}");

            while let Some((key, value)) = map.pop_first() {
                content += &format!(",\"{key}\":{value:?}")
            }

            content
        })
    }
}