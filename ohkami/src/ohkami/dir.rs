#![cfg(feature="__rt_native__")]

use crate::handler::{Handler, IntoHandler};
use crate::response::Content;
use std::fs::File;
use std::path::{PathBuf, Path};

pub struct Dir {
    pub(crate) route: &'static str,
    pub(crate) files: Vec<(PathBuf, File)>,

    /*=== config ===*/
    pub(crate) serve_dotfiles: bool,
    pub(crate) omit_extensions: &'static [&'static str],
}

impl Dir {
    pub(super) fn new(route: &'static str, dir_path: std::path::PathBuf) -> std::io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} is not directory", dir_path.display()))
            )
        }

        let mut files = Vec::<(PathBuf, File)>::new(); {
            fn fetch_entries(
                dir: std::path::PathBuf
            ) -> std::io::Result<Vec<std::path::PathBuf>> {
                dir.read_dir()?
                    .map(|de| de.map(|de| de.path()))
                    .collect()
            }

            let mut entries = fetch_entries(dir_path.clone())?;
            while let Some(entry) = entries.pop() {
                if entry.is_file() {
                    files.push((
                        entry.iter().skip(dir_path.iter().count()).collect(),
                        std::fs::File::open(entry)?
                    ));

                } else if entry.is_dir() {
                    entries.extend(fetch_entries(entry)?);

                } else {
                    continue
                }
            }
        }

        Ok(Self {
            route,
            files,
            serve_dotfiles: false,
            omit_extensions: &[],
        })
    }

    /// Whether to serve dotfiles like `.abc` (default: false)
    /// 
    /// When `false`, files of name starting with `.` will be ignored.
    pub fn serve_dotfiles(mut self, yes: bool) -> Self {
        self.serve_dotfiles = yes;
        self
    }

    /// File extensions (leading `.` trimmed) that should not be appeared in server path.
    /// For example, if you omit `[".html"]`, `/abc.html` will be served at `/abc` instead of `/abc.html`.
    /// 
    /// As a special case, `index.html` will be served at `/` when `".html"` is omitted.
    pub fn omit_extensions(mut self, extensions_to_omit: &'static [&'static str]) -> Self {
        self.omit_extensions = extensions_to_omit;
        self
    }
}

#[derive(Clone)]
pub(super) struct StaticFileHandler {
    mime:    &'static str,
    content: std::sync::Arc<Vec<u8>>,
}

impl StaticFileHandler {
    pub(super) fn new(path: &Path, file: std::fs::File) -> std::io::Result<Self> {
        let mime = ::mime_guess::from_path(path)
            .first_raw()
            .unwrap_or("application/octet-stream");

        let mut content = vec![
            u8::default();
            file.metadata().unwrap().len() as usize
        ]; {use std::io::Read;
            let mut file = file;
            file.read_exact(&mut content)?;
        }

        Ok(Self { mime, content:std::sync::Arc::new(content) })
    }
}

impl IntoHandler<File> for StaticFileHandler {
    fn n_params(&self) -> usize {0}

    fn into_handler(self) -> Handler {
        let this: &'static StaticFileHandler
            = Box::leak(Box::new(self));

        Handler::new(|_| Box::pin(async {
            let mut res = crate::Response::OK();
            {
                let content: &'static [u8] = &this.content;
                res.headers.set()
                    .ContentType(this.mime)
                    .ContentLength(ohkami_lib::num::itoa(content.len()));
                res.content = Content::Payload(content.into());
            }
            res
        }), #[cfg(feature="openapi")] {use crate::openapi;
            openapi::Operation::with(openapi::Responses::new([(
                200,
                openapi::Response::when("OK")
                    .content(this.mime, openapi::string().format("binary"))
            )]))
        })
    }
}
