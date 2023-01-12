use quote::quote;

use crate::internals::Build;

use super::JsonStr;

impl Build for JsonStr {
    fn build(self) -> proc_macro2::TokenStream {
        let mut str = format!("{self:?}");
        quote!(#str)
    }
}