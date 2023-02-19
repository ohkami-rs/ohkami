// pub fn ser<S: serde::Serialize>(s: &S) -> Result<String> {
//     Ok(serde_json::to_string(s)?)
// }
// pub fn de<'de, D: serde::Deserialize<'de>>(str: &'de str) -> Result<D> {
//     Ok(serde_json::from_str(str)?)
// }

use serde::Serialize;

pub trait JSON<'j>: serde::Serialize + serde::Deserialize<'j> {
    fn ser(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    fn de(string: &'j str) -> crate::Result<Self> {
        Ok(serde_json::from_str(string)?)
    }
}
impl <'i, J: for <'j> JSON<'j>> JSON<'i> for Vec<J> {}


pub trait JsonResponse<L: JsonResponseLabel>: Serialize {fn ser(&self) -> crate::Result<String>;}
pub trait JsonResponseLabel {}

impl JsonResponseLabel for () {}
impl<J: for <'j> JSON<'j>> JsonResponse<()> for J {
    fn ser(&self) -> crate::Result<String> {
        self.ser()
    }
}
impl JsonResponseLabel for &() {}
impl<J: for <'j> JSON<'j>> JsonResponse<&()> for &J {
    fn ser(&self) -> crate::Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}


#[cfg(test)]
mod test {
    use serde::{Serialize, Deserialize};
    use crate::prelude::Response;
    use super::JSON;

    #[derive(Serialize, Deserialize)]
    struct User {
        id:   u64,
        name: String,
    } impl<'j> JSON<'j> for User {}

    fn _ref_json() {
        let user = User {id: 1, name: String::from("Taro")};
        let _ = Response::OK(&user);
    }
}