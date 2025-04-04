#![cfg(feature="__rt_native__")]

use crate::handler::{Handler, IntoHandler};
use crate::header::{ETag, Encoding, CompressionEncoding, AcceptEncoding};
use ohkami_lib::{time::ImfFixdate, map::TupleMap};
use std::{io, fs::File};
use std::path::{PathBuf, Path};

pub struct Dir {
    pub(super) route: &'static str,
    pub(super) files: TupleMap<PathBuf, Vec<StaticFile>>,

    /*=== config ===*/
    pub(super) serve_dotfiles: bool,
    pub(super) omit_extensions: &'static [&'static str],
    pub(super) etag: Option<fn(&File) -> String>,
}

pub(super) enum StaticFile {
    Source {
        file: File,
    },
    Compressed {
        encoding: CompressionEncoding,
        file: File,
    },
}
impl StaticFile {
    fn new(path: &Path) -> io::Result<(Self, std::borrow::Cow<'_, Path>)> {
        let file = File::open(path)?;
        match CompressionEncoding::from_file_path(path) {
            None => Ok((
                Self::Source { file },
                path.into()
            )),
            Some((encoding, source)) => Ok((
                Self::Compressed {
                    encoding,
                    file,
                },
                source.into()
            ))
        }
    }

    fn into_file(self) -> File {
        match self {
            Self::Source { file } => file,
            Self::Compressed { file, .. } => file,
        }
    }

    fn is_source(&self) -> bool {
        match self {
            Self::Source { .. } => true,
            Self::Compressed { .. } => false,
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
                    let source_path = source_path
                        .canonicalize()?
                        .strip_prefix(&dir_path)
                        .map_err(|_| io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("{} is not a child of {}", path.display(), dir_path.display())
                        ))?
                        .to_owned();
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

fn read(mut file: File) -> io::Result<Vec<u8>> {
    let mut content = vec![0; file.metadata()?.len() as usize];    
    io::Read::read_exact(&mut file, &mut content)?;
    Ok(content)
}

fn cmp_size(f: &File, g: &File) -> std::cmp::Ordering {
    u64::cmp(
        &f.metadata().map(|m| m.len()).unwrap_or(u64::MAX),
        &g.metadata().map(|m| m.len()).unwrap_or(u64::MAX)
    )
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
    content: Vec<u8>,
    compressed: Vec<(CompressionEncoding, Vec<u8>)>,
}

impl StaticFileHandler {
    pub(super) fn new(
        path: &Path,
        mut files: Vec<StaticFile>,
        get_etag: Option<fn(&File) -> String>,
    ) -> io::Result<Self> {
        let (source_file, compressed_files) = {
            let source = files.swap_remove(
                files
                .iter()
                .position(StaticFile::is_source)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("No source file found for {}", path.display())
                ))?
            ).into_file();
            
            // Check `files` contains single source file
            if files.iter().any(|f| f.is_source()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Multiple source files found for {}", path.display())
                ));
            }

            let mut compressed = files
                .into_iter()
                .map(|f| {
                    // here `files` is guaranteed to have only compressed files
                    let StaticFile::Compressed { encoding, file } = f else {unreachable!()};
                    (encoding, file)
                })
                .collect::<Vec<_>>();

            // Put priority on how small each compressed file is
            compressed.sort_unstable_by(|(_, f), (_, g)| cmp_size(f, g));

            (source, compressed)
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
            // Check if the compressed file is not older than the source file
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

            let content = read(file)?;
            compressed.push((encoding, content));
        }

        crate::DEBUG!(
            "[Dir] precompressed files for {}: {:?}",
            path.display(),
            compressed.iter().map(|(ce, _)| ce).collect::<Vec<_>>()
        );

        Ok(Self {
            last_modified,
            last_modified_str,
            etag,
            mime,
            compressed,
            content: read(source_file)?,
        })
    }
}

impl IntoHandler<File> for StaticFileHandler {
    fn n_params(&self) -> usize {0}

    fn into_handler(self) -> Handler {
        let this = Box::leak(Box::new(self));

        Handler::new(|req| Box::pin(async {
            use crate::Response;

            // Check if client's cache is still valid
            // and then return 304 Not Modified.
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

            // Check if client's Accept-Encoding header matches the available encodings
            // and determine which encoding to use.
            // 
            // If no matching encoding is found, try falling back to the original content.
            // Then, if client doesn't accept identity encoding, return 406 Not Acceptable
            // instead of returning the original content.
            let (encoding, content) = {
                let ae = req
                    .headers
                    .AcceptEncoding()
                    .map(AcceptEncoding::parse)
                    .unwrap_or_default();

                crate::DEBUG!("[Dir] Accept-Encoding: {:?}", ae);
                crate::DEBUG!("[Dir] precompressed canidadates: {:?}", this.compressed.iter().map(|(ce, _)| ce).collect::<Vec<_>>());

                if let Some((encoding, content)) = this
                    .compressed
                    .iter()
                    .find(|(ce, _)| ae.accepts_compression(ce))
                {
                    crate::DEBUG!("[Dir] using precompressed: {:?}", encoding);
                    (Some(encoding), &**content)

                } else if ae.accepts(Encoding::Identity) {
                    (None, &*this.content)

                } else {
                    return Response::NotAcceptable();
                }
            };

            Response::OK()
                .with_payload(this.mime, content)
                .with_headers(|h| h
                    .LastModified(&*this.last_modified_str)
                    .ETag(this.etag.as_ref().map(ETag::serialize))
                    .ContentEncoding(encoding.map(CompressionEncoding::to_content_encoding))
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
