use std::{collections::BTreeMap, fmt::Debug};
use super::JsonStr;

pub(super) struct Object(
    pub(crate) BTreeMap<&'static str, JsonStr>
); impl Object {
    fn var_indexes(&self) -> Vec<usize> {
        let mut indexes = Vec::new();
        for (i, (key, value)) in self.0.iter().enumerate() {
            match value {
                JsonStr::Var(_) => indexes.push(i),
                _ => (),
            }
        }
        indexes
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let map = self.0;
        let len = map.len();

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