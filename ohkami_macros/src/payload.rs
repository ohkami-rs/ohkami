use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::{Result, ItemStruct};

use crate::components::*;


#[allow(non_snake_case)]
pub(super) fn Payload(format: TokenStream, data: TokenStream) -> Result<TokenStream> {
    let format = Format::parse(format)?;
    let data = parse_struct("Payload", data)?;

    let impl_payload = match format {
        Format::JSON => impl_payload_json(&data),
        Format::Form => impl_payload_form(&data),
    }?;

    Ok(quote!{
        #data
        #impl_payload
    })
}

fn impl_payload_json(data: &ItemStruct) -> Result<TokenStream> {

}

fn impl_payload_form(data: &ItemStruct) -> Result<TokenStream> {
    
}
