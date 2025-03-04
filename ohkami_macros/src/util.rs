#![allow(unused)]

use std::{fs::File, path::Path};
use std::io::{self, ErrorKind};
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

#[cfg(feature="worker")]
/// When a proc macro is called in a workspace, current Rust handles relative paths
/// of accesses to file system as if it's *relative from the workspace root* and
/// the proc macro isn't aware of which package called it.
/// 
/// So, if found the file by simple reading of `file_path`, this returns the file,
/// but if not found, this assumes a Cargo workspace and search all `workspace.members`
/// to find one having file at `file_path`.
pub(crate) fn find_file_at_package_or_workspace_root(file_path: impl AsRef<Path>) -> Result<Option<File>, io::Error> {
    let file_path: &Path = file_path.as_ref();

    match File::open(file_path) {
        Ok(file) => {
            Ok(Some(file))
        }
        Err(e) if matches!(e.kind(), ErrorKind::NotFound) => {
            find_file_at_workspace_root(file_path)
        }
        Err(e) => {
            Err(e)
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////

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

#[cfg(feature="worker")]
fn find_file_at_workspace_root(file_path: impl AsRef<Path>) -> Result<Option<File>, io::Error> {
    let file_path: &Path = file_path.as_ref();

    let cargo_toml: toml::Value = {use std::io::Read;
        let mut file = File::open("Cargo.toml")?;
        let mut buf  = String::new();
        file.read_to_string(&mut buf)?;
        toml::from_str(&buf).expect("Invalid Cargo.toml")
    };

    fn get_workspace_members(cargo_toml: &toml::Value) -> Option<&toml::value::Array> {
        cargo_toml
            .as_table()?
            .get("workspace")?
            .as_table()?
            .get("members")?
            .as_array()
    }

    let Some(workspace_members) = get_workspace_members(&cargo_toml) else {
        return Ok(None)
    };

    let mut matching_files = Vec::with_capacity(1);
    for member in workspace_members {
        if let Ok(file) = File::open(Path::new(member.as_str().unwrap()).join(file_path)) {
            matching_files.push(file)
        }
    }

    match (matching_files.pop(), matching_files.is_empty()) {
        (Some(file), true) => {
            Ok(Some(file))
        }
        (None, _) => {
            Ok(None)
        }
        (Some(_), false) => {
            Err(io::Error::new(ErrorKind::Other, format!(
                "Multiple workspace members have `{}`, this is not supported", file_path.display()
            )))
        }
    }
}
