#![cfg(feature="__rt_native__")]

use crate::handler::{Handler, IntoHandler};
use crate::header::{ETag, Encoding, CompressionEncoding, AcceptEncoding, QValue};
use ohkami_lib::{time::ImfFixdate, map::TupleMap};
use std::{io, fs::File};
use std::path::{PathBuf, Path};

pub struct Dir {
    pub(crate) route: &'static str,
    pub(crate) files: TupleMap<PathBuf, Vec<StaticFile>>,

    /*=== config ===*/
    pub(crate) serve_dotfiles: bool,
    pub(crate) omit_extensions: &'static [&'static str],
    pub(crate) etag: Option<fn(&File) -> String>,
}

enum StaticFile {
    Source {
        file: File,
    },
    Compression {
        encodings: CompressionEncoding,
        file: File,
    },
}
impl StaticFile {
    fn new(path: &Path) -> io::Result<(Self, &Path)> {
        let file = File::open(path)?;
        match CompressionEncoding::from_file_path(path) {
            None => Ok((
                Self::Source { file },
                path
            )),
            Some((encodings, source)) => Ok((
                Self::Compression {
                    encodings,
                    file,
                },
                source
            ))
        }
    }

    fn into_file(self) -> File {
        match self {
            Self::Source { file } => file,
            Self::Compression { file, .. } => file,
        }
    }

    fn is_source(&self) -> bool {
        match self {
            Self::Source { .. } => true,
            Self::Compression { .. } => false,
        }
    }
}

impl Dir {
    pub(super) fn new(route: &'static str, dir_path: PathBuf) -> io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a directory", dir_path.display()))
            )
        }

        let mut files = TupleMap::<PathBuf, Vec<StaticFile>>::new(); {
            fn fetch_entries(
                dir: PathBuf
            ) -> io::Result<Vec<PathBuf>> {
                dir.read_dir()?
                    .map(|de| de.map(|de| de.path()))
                    .collect()
            }

            let mut entries = fetch_entries(dir_path.clone())?;
            while let Some(path) = entries.pop() {
                if path.is_file() {
                    let (file, source_path) = StaticFile::new(&path)?;
                    let source_path = source_path.to_owned();
                    if let Some(them) = files.get_mut(&source_path) {
                        them.push(file);
                    } else {
                        files.insert(source_path, vec![file]);
                    }

                } else if path.is_dir() {
                    entries.extend(fetch_entries(path)?);

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

type StaticFileContent = &'static [u8];

fn static_file_content(mut file: File) -> io::Result<StaticFileContent> {
    let mut content = vec![0; file.metadata()?.len() as usize];    
    io::Read::read_exact(&mut file, &mut content)?;
    Ok(content.leak())
}

fn modified_unix_timestamp(file: &File) -> io::Result<u64> {
    let ts = file
        .metadata()?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(ts)
}

pub(super) struct StaticFileHandler {
    last_modified: ImfFixdate,
    last_modified_str: String,
    etag: Option<ETag<'static>>,
    mime: &'static str,
    content: StaticFileContent,
    compressed: Vec<(CompressionEncoding, StaticFileContent)>,
}

impl StaticFileHandler {
    pub(super) fn new(
        path: &Path,
        mut files: Vec<StaticFile>,
        get_etag: Option<fn(&File) -> String>,
    ) -> io::Result<Self> {
        let (source_file, compressed_files) = {
            let s = files.iter().position(StaticFile::is_source)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("No source file found for {}", path.display())
                ))?;
            
            if files.iter().any(|f| f.is_source()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Multiple source files found for {}", path.display())
                ));
            }

            (
                files.swap_remove(s).into_file(),
                files.into_iter().map(|f| {
                    let StaticFile::Compression { encodings, file } = f else {unreachable!()};
                    (encodings, file)
                })
            )
        };

        let last_modified_str = ohkami_lib::time::UTCDateTime::from_unix_timestamp(
            modified_unix_timestamp(&source_file)?
        ).into_imf_fixdate();

        let last_modified =  ImfFixdate::parse(&last_modified_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let etag = get_etag
            .map(|f| ETag::new(f(&source_file)))
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mime = ::mime_guess::from_path(&path)
            .first_raw()
            .unwrap_or("application/octet-stream");

        let mut compressed = Vec::new();
        for (encoding, file) in compressed_files {
            // Check if the compressed file is newer than the source file
            // because the compressed file may be generated from the source file
            // and we want to avoid serving an outdated compressed file.
            if modified_unix_timestamp(&file)? < modified_unix_timestamp(&source_file)? {
                crate::WARNING!(
                    "[Dir] skipping outdated compressed file {}.{}: older than source file {}",
                    path.display(), encoding.to_extension(),
                    path.display()
                );
                continue
            }

            let content = static_file_content(file)?;
            compressed.push((encoding, content));
        }

        Ok(Self {
            last_modified,
            last_modified_str,
            etag,
            mime,
            compressed,
            content: static_file_content(source_file)?,
        })
    }

    fn register_compression(
        &mut self,
        encoding: CompressionEncoding,
        content: StaticFileContent,
    ) {
        self.compressed.push((encoding, content));
    }
}

impl IntoHandler<File> for StaticFileHandler {
    fn n_params(&self) -> usize {0}

    fn into_handler(self) -> Handler {
        let this: &'static StaticFileHandler = Box::leak(Box::new(self));

        Handler::new(|req| Box::pin(async {
            use crate::Response;

            // Check if the client's cache is still valid
            // and then return 304 Not Modified
            if let (Some(if_none_match), Some(etag)) = (req.headers.IfNoneMatch(), &this.etag) {
                if ETag::iter_from(if_none_match).any(|it| it.matches(etag)) {
                    return Response::NotModified();
                }
            } else if let Some(if_modified_since) = req.headers.IfModifiedSince() {
                let Ok(if_modified_since) = ImfFixdate::parse(if_modified_since) else {
                    return Response::BadRequest();
                };
                if if_modified_since >= this.last_modified {
                    return Response::NotModified();
                }
            }

            // let accept_encoding = req.headers.AcceptEncoding()
            //     .map(AcceptEncoding::parse)
            //     .unwrap_or_default();
            // if !/*not*/ this.encodings
            //     .as_deref()
            //     .unwrap_or(&[Encoding::Identity])
            //     .iter()
            //     .all(|e| accept_encoding.accepts(*e))
            // {
            //     TODO
            // }

            Response::OK()
                .with_payload(this.mime, &*this.content)
                .with_headers(|h| h
                    .LastModified(&*this.last_modified_str)
                    .ETag(this.etag.as_ref().map(ETag::serialize))
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
