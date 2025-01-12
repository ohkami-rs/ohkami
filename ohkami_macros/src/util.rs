#![allow(unused)]

use syn::{Expr, ExprLit, Lit, Attribute};

#[allow(non_snake_case)]
pub(crate) fn is_Option(ty: &syn::Type) -> bool {
    let ty = quote::ToTokens::to_token_stream(ty).to_string();
    ty.ends_with(" >") && (
        ty.starts_with("Option < ") ||
        ty.starts_with("std::option::Option < ") ||
        ty.starts_with("core::option::Option < ") ||
        ty.starts_with("::std::option::Option < ") ||
        ty.starts_with("::core::option::Option < ")
    )
}

#[allow(non_snake_case)]
pub(crate) fn inner_Option(ty: &syn::Type) -> Option<syn::Type> {
    let ty = quote::ToTokens::to_token_stream(ty).to_string();
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

pub(crate) fn extract_doc_attrs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs.iter()
        .filter(|a| a.meta.require_name_value().is_ok_and(
            |m| m.path.get_ident().is_some_and(|i| i == "doc")
        ))
        .map(Attribute::clone)
        .collect()
}

pub(crate) fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc = String::new();

    for a in attrs.iter()
        .filter_map(|a| a.meta.require_name_value().ok())
        .filter(|m| m.path.get_ident().is_some_and(|i| i == "doc"))
    {
        let Expr::Lit(ExprLit { lit: Lit::Str(raw_doc), .. }) = &a.value else {continue};
        if !doc.is_empty() {
            doc.push('\n')
        }
        doc.push_str(unescaped_doc_attr(raw_doc.value()).trim());
    }

    (!doc.is_empty()).then_some(doc)
}

fn unescaped_doc_attr(raw_doc: String) -> String {
    let mut unescaped = String::with_capacity(raw_doc.len());
    {
        let mut chars = raw_doc.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
                /* do nothing to unescape the next charactor */
            } else {
                unescaped.push(ch);
            }
        }
    }
    unescaped
}
