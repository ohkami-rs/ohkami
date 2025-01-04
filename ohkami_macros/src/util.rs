#[allow(non_snake_case)]
pub(crate) fn is_Option(&ty: syn::Type) -> bool {
    let ty = ty.to_token_stream().to_string();
    ty.ends_with(" >") && (
        ty.starts_with("Option < ") ||
        ty.starts_with("std::option::Option < ") ||
        ty.starts_with("core::option::Option < ") ||
        ty.starts_with("::std::option::Option < ") ||
        ty.starts_with("::core::option::Option < ")
    )
}

#[allow(non_snake_case)]
pub(crate) fn inner_Option(&ty: syn::Type) -> Option<syn::Type> {
    let ty = ty.to_token_stream().to_string();
    match ty.split_once(" < ")? {
        (
            | "Option"
            | "std::option::Option"
            | "core::option::Option"
            | "::std::option::Option"
            | "::core::option::Option",
            remained
        ) => {
            syn::parse_str(remained.strip_suffix(" >")?).ok()
        }
        _ => None
    }
}
