mod status;
pub(crate) mod ok;
pub(crate) mod err;
pub(crate) mod body;
pub(crate) mod header;

use serde::Serialize;
use std::ops::{ControlFlow, Try, FromResidual};
use self::{ok::OkResponse, err::ErrResponse};

pub enum Response<T: Serialize> {
    Ok(OkResponse<T>),
    Err(ErrResponse),
}

impl<T: Serialize> Try for Response<T> {
    type Residual = ErrResponse;
    type Output = OkResponse<T>;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Err(err_res) => ControlFlow::Break(err_res),
            Self::Ok(ok_res) => ControlFlow::Continue(ok_res),
        }
    }
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }
}

impl<T: Serialize> FromResidual<ErrResponse> for Response<T> {

}
