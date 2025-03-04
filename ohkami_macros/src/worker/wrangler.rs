use crate::util;
use std::io::{self, Read};

pub fn parse_wrangler<T: serde::de::DeserializeOwned>() -> Result<T, std::io::Error> {
        fn parse_error(e: impl std::fmt::Display) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse wrangler config: `{e}`")
        )
    }

    let mut buf = String::new();

    match (
        util::find_file_at_package_or_workspace_root("wrangler.toml")?,
        util::find_file_at_package_or_workspace_root("wrangler.jsonc")?
    ) {
        (Some(_), Some(_)) => {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "both `wrangler.toml` and `wrangler.jsonc` is found !"
            ))
        }
        (None, None) => {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "neither `wrangler.toml` nor `wrangler.jsonc` is found at package or workspace root"
            ))
        }
        (Some(mut wrangler_toml), None) => {
            wrangler_toml.read_to_string(&mut buf)?;
            let config = toml::from_str(&buf)
                .map_err(parse_error)?;
            Ok(config)
        }
        (None, Some(mut wrangler_jsonc)) => {
            wrangler_jsonc.read_to_string(&mut buf)?;
            let config = jsonc_parser::parse_to_serde_value(&buf, &Default::default())
                .map_err(parse_error)?
                .ok_or_else(|| parse_error("invalid `.jsonc`"))?;
            let config = serde_json::from_value(config)
                .map_err(parse_error)?;
            Ok(config)
        }
    }
}