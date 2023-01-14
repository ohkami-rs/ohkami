use crate::prelude::Result;

pub fn ser<S: serde::Serialize>(s: &S) -> Result<String> {
    Ok(serde_json::to_string(s)?)
}
pub fn de<'de, D: serde::Deserialize<'de>>(str: &'de str) -> Result<D> {
    Ok(serde_json::from_str(str)?)
}

pub trait Json<'j>: serde::Serialize + serde::Deserialize<'j> {
    fn ser(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    fn de(string: &'j str) -> Result<Self> {
        Ok(serde_json::from_str(string)?)
    }
}
impl <'i, J: for <'j> Json<'j>> Json<'i> for Vec<J> {}


pub trait JsonResponse<L: JsonResponseLabel> {fn ser(&self) -> Result<String>;}
pub trait JsonResponseLabel {}

impl JsonResponseLabel for () {}
impl<J: for <'j> Json<'j>> JsonResponse<()> for J {
    fn ser(&self) -> Result<String> {
        self.ser()
    }
}
impl JsonResponseLabel for &() {}
impl<J: for <'j> Json<'j>> JsonResponse<&()> for &J {
    fn ser(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}


#[cfg(test)]
mod test {
    use serde::{Serialize, Deserialize};
    use crate::prelude::Response;
    use super::Json;

    #[derive(Serialize, Deserialize)]
    struct User {
        id:   u64,
        name: String,
    } impl<'j> Json<'j> for User {}

    fn _ref_json() {
        let user = User {id: 1, name: String::from("Taro")};
        let _ = Response::OK(&user);
    }
}