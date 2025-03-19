#![cfg(feature="__rt_native__")]

use crate::handler::{Handler, IntoHandler};
use crate::header::ETag;
use ohkami_lib::time::ImfFixdate;
use std::{io, fs::File};
use std::path::{PathBuf, Path};

pub struct Dir {
    pub(crate) route: &'static str,
    pub(crate) files: Vec<(PathBuf, File)>,

    /*=== config ===*/
    pub(crate) serve_dotfiles: bool,
    pub(crate) omit_extensions: &'static [&'static str],
    pub(crate) etag: Option<fn(&File) -> String>,
}

impl Dir {
    pub(super) fn new(route: &'static str, dir_path: PathBuf) -> io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not directory", dir_path.display()))
            )
        }

        let mut files = Vec::<(PathBuf, File)>::new(); {
            fn fetch_entries(
                dir: PathBuf
            ) -> io::Result<Vec<PathBuf>> {
                dir.read_dir()?
                    .map(|de| de.map(|de| de.path()))
                    .collect()
            }

            let mut entries = fetch_entries(dir_path.clone())?;
            while let Some(entry) = entries.pop() {
                if entry.is_file() {
                    files.push((
                        entry.iter().skip(dir_path.iter().count()).collect(),
                        File::open(entry)?
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
            etag: None,
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

    /// Set a function to generate ETag for each file.
    pub fn etag(mut self, etag: impl Into<Option<fn(&File) -> String>>) -> Self {
        self.etag = etag.into();
        self
    }
}

#[derive(Clone)]
pub(super) struct StaticFileHandler {
    last_modified: ImfFixdate,
    last_modified_str: String,
    etag: Option<ETag<'static>>,
    mime: &'static str,
    content: std::sync::Arc<Vec<u8>>,
}

impl StaticFileHandler {
    pub(super) fn new(
        path: &Path,
        file: File,
        get_etag: Option<fn(&File) -> String>,
    ) -> io::Result<Self> {
        let last_modified_str = ohkami_lib::time::UTCDateTime::from_unix_timestamp(
            file
            .metadata()?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
        ).into_imf_fixdate();

        let last_modified =  ImfFixdate::parse(&last_modified_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let etag = get_etag
            .map(|f| ETag::new(f(&file)))
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mime = ::mime_guess::from_path(path)
            .first_raw()
            .unwrap_or("application/octet-stream");

        let mut content = vec![
            u8::default();
            file.metadata().unwrap().len() as usize
        ]; {use io::Read;
            let mut file = file;
            file.read_exact(&mut content)?;
        }

        Ok(Self {
            last_modified,
            last_modified_str,
            etag,
            mime,
            content: std::sync::Arc::new(content)
        })
    }
}

impl IntoHandler<File> for StaticFileHandler {
    fn n_params(&self) -> usize {0}

    fn into_handler(self) -> Handler {
        let this: &'static StaticFileHandler = Box::leak(Box::new(self));

        Handler::new(|req| Box::pin(async {
            use crate::{Response, header::ETag};

            if let (Some(if_none_match), Some(etag)) = (req.headers.IfNoneMatch(), &this.etag) {
                if ETag::iter_from(if_none_match).any(|it| it.matches(etag)) {
                    return Response::NotModified();
                }
            }
            if let Some(if_modified_since) = req.headers.IfModifiedSince() {
                let Ok(if_modified_since) = ImfFixdate::parse(if_modified_since) else {
                    return Response::BadRequest();
                };
                if if_modified_since >= this.last_modified {
                    return Response::NotModified();
                }
            }

            Response::OK()
                .with_payload(this.mime, &*this.content)
                .with_headers(|h| h
                    .LastModified(&*this.last_modified_str)
                    .ETag(this.etag.as_ref().map(|etag| etag.serialize()))
                )
        }), #[cfg(feature="openapi")] {use crate::openapi;
            openapi::Operation::with(openapi::Responses::new([
                (
                    200,
                    openapi::Response::when("OK")
                        .content(this.mime, openapi::string().format("binary"))
                ),
                (
                    304,
                    openapi::Response::when("Not Modified")
                )
            ]))
        })
    }
}
