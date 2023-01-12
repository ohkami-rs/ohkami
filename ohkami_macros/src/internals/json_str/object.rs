use std::{collections::BTreeMap, fmt::Debug};

use super::JsonStr;

pub(super) struct Map(
    BTreeMap<&'static str, JsonStr>
);

impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", 'content: {
            let mut content = String::new();
            let mut map = self.0;

            let Some((first_key, first_value)) = map.pop_first() else {
                break 'content content
            };
            content += &format!("{first_key}:{first_value:?}");

            while let Some((key, value)) = map.pop_first() {
                content += &format!(",{key}:{value:?}")
            }

            content
        })
    }
}