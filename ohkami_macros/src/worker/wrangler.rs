use crate::util;
use std::io::{self, Read};

pub fn parse_wrangler<T: serde::de::DeserializeOwned>() -> Result<T, std::io::Error> {
        fn parse_error(filename: &str, e: impl std::fmt::Display) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse `{filename}`: `{e}`")
        )
    }

    let mut buf = String::new();

    if let Some(mut wrangler_toml) = util::find_file_at_package_or_workspace_root("wrangler.toml")? {
        wrangler_toml.read_to_string(&mut buf)?;
        let config = toml::from_str(&buf)
            .map_err(|e| parse_error("wrangler.toml", e))?;
        Ok(config)

    } else if let Some(mut wrangler_jsonc) = util::find_file_at_package_or_workspace_root("wrangler.jsonc")? {
        wrangler_jsonc.read_to_string(&mut buf)?;
        let config = jsonc_parser::parse_to_serde_value(&buf, &Default::default())
            .map_err(|e| parse_error("wrangler.jsonc", e))?
            .ok_or_else(|| parse_error("wrangler.jsonc", "invalid `.jsonc`"))?;
        let config = serde_json::from_value(config)
            .map_err(|e| parse_error("wrangler.jsonc", e))?;
        Ok(config)

    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "neither `wrangler.toml` nor `wrangler.jsonc` is found at package or workspace root"
        ))
    }
}