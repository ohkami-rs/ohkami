pub use crate::header::ETag;
use crate::header::{AcceptEncoding, CompressionEncoding, Encoding};
use ohkami_lib::time::{ImfFixdate, UTCDateTime};
use std::fs::{File, Metadata};
use std::path::{Path, PathBuf};

const DEFAULT_BUFFER_SIZE: usize = 64 * (1 << 10); // 64 KiB

struct StaticFileHandler {
    file_path: PathBuf,
    buffer_size: usize,
    etag_fn: fn(&File) -> Option<ETag<'static>>,
    mime_type: &'static str,
    last_modified: std::sync::RwLock<LastModified>,
    etag_cache: Option<std::sync::RwLock<ETag<'static>>>,
    full_buffer: Option<Box<[u8]>>,
}

struct LastModified {
    unix_timestamp: u64,
    imf_fixdate: ImfFixdate,
}

impl StaticFileHandler {
    fn from_servefile(
        ServeFile {
            file_path,
            buffer_size,
            etag_fn,
            full_buffering,
        }: ServeFile,
    ) -> Self {
        let mime_type = ::mime_guess::from_path(&file_path)
            .first_raw()
            .unwrap_or("application/octet-stream");

        let file = File::open(&file_path).unwrap();

        let etag_cache = etag_fn(&file).map(std::sync::RwLock::new);

        let full_buffer = if full_buffering {
            let mut buf = Vec::new();
            std::io::copy(&mut &file, &mut buf).unwrap();
            Some(buf.into_boxed_slice())
        } else {
            None
        };

        let last_modified = std::sync::RwLock::new({
            let unix_timestamp = file
                .metadata()
                .unwrap()
                .modified()
                .unwrap()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            LastModified {
                unix_timestamp,
                imf_fixdate: ImfFixdate::from_unix_timestamp(unix_timestamp),
            }
        });

        Self {
            file_path,
            buffer_size,
            etag_fn,
            mime_type,
            last_modified,
            etag_cache,
            full_buffer,
        }
    }
}

impl crate::fang::handler::IntoHandler<File> for StaticFileHandler {
    fn n_pathparams(&self) -> usize {
        0
    }

    fn into_handler(self) -> crate::handler::Handler {
        let handlers_for_all_available_encodings = 'search: {
            let buffer_size = self.buffer_size;            
            let self_file_path = self.file_path.clone();
            let self_etag_fn = self.etag_fn;
            let self_full_buffering = self.full_buffer.is_some();

            let Ok(dir_entries) = self_file_path.parent().unwrap().read_dir() else {
                break 'search vec![(CompressionEncoding::Single(Encoding::Identity), self)];
            };
            dir_entries
                .filter_map(|r| {
                    r.ok().and_then(|entry| {
                        let path = entry.path();
                        match CompressionEncoding::from_file_path(&path) {
                            Some((encoding, source)) if source == self_file_path => {
                                Some((
                                    encoding,
                                    StaticFileHandler::from_servefile(ServeFile {
                                        buffer_size,
                                        file_path: path,
                                        etag_fn: self_etag_fn,
                                        // fullly buffer only if the original file is fully buffered.
                                        // (assuming that compressed files are also small and immutable as original file)
                                        full_buffering: self_full_buffering,
                                    }),
                                ))
                            }
                            _ => None,
                        }
                    })
                })
                .chain([(CompressionEncoding::Single(Encoding::Identity), self)])
                .collect::<Vec<_>>()
        };

        crate::handler::Handler::new(|req| Box::pin(async {
            
            
            todo!()
        }))
    }
}

pub struct ServeFile {
    file_path: PathBuf,
    buffer_size: usize,
    etag_fn: fn(&File) -> Option<ETag<'static>>,
    full_buffering: bool,
}

impl ServeFile {
    /// Create a new `ServeFile` instance to serve a specific file.
    ///
    /// ## Panics
    ///
    /// This panics if:
    ///
    /// - the provided path is not a valid file path.
    /// - the file is not accessible.
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        let file_path = file_path.as_ref().to_path_buf();
        assert!(
            file_path.is_file() && File::open(&file_path).is_ok(),
            "Provided path is not a valid or accessible file: {}",
            file_path.display()
        );

        fn no_etag(_: &File) -> Option<ETag<'static>> {
            None
        }

        Self {
            file_path,
            buffer_size: DEFAULT_BUFFER_SIZE,
            etag_fn: no_etag,
            full_buffering: false,
        }
    }

    /// Configure the buffer size [bytes] used when serving the file.
    ///
    /// ## Default
    ///
    /// `64 * 1024` (64 KiB)
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Configure a function to generate ETag for the file served.
    ///
    /// ## Default
    ///
    /// `None` (no ETag is generated)
    pub fn etag_fn(mut self, func: fn(&File) -> Option<ETag<'static>>) -> Self {
        self.etag_fn = func;
        self
    }

    /// Configure whether to fully buffer the file into memory before serving.
    ///
    /// ## Default
    ///
    /// `false`
    ///
    /// ## Note
    ///
    /// Should be set to `true` only if the file is **small** and **immutable**.
    ///
    /// ### Pros
    /// - Reduces disk I/O during serving, potentially improving performance for frequently accessed files.
    /// - Enables faster response times for clients, especially for small to medium-sized files.
    ///
    /// ### Cons
    /// - Increases memory usage, which may be a concern for large files or high traffic scenarios.
    /// - Disables to follow file changes on disk after the initial load.
    pub fn full_buffering(mut self, yes: bool) -> Self {
        self.full_buffering = yes;
        self
    }
}

pub struct ServeDir {
    pub(super) dir_path: PathBuf,
    pub(super) buffer_size: usize,
    pub(super) serve_dotfiles: bool,
    pub(super) omit_extensions: Vec<&'static str>,
    pub(super) etag_fn: fn(&Path) -> Option<ETag<'static>>,
    pub(super) full_buffering_fn: fn(&Path) -> bool,
}

impl ServeDir {
    /// Create a new `ServeDir` instance to serve files from a directory.
    ///
    /// ## Panics
    ///
    /// This panics if:
    ///
    /// - the provided path is not a valid directory path.
    /// - the directory is not accessible.
    pub fn new(dir_path: impl AsRef<Path>) -> Self {
        let dir_path = dir_path.as_ref().to_path_buf();
        assert!(
            dir_path.read_dir().is_ok(),
            "Provided path is not a valid or accessible directory: {}",
            dir_path.display()
        );

        fn no_etag(_: &Path) -> Option<ETag<'static>> {
            None
        }

        fn no_full_buffering(_: &Path) -> bool {
            false
        }

        Self {
            dir_path,
            buffer_size: DEFAULT_BUFFER_SIZE,
            serve_dotfiles: false,
            omit_extensions: vec![],
            etag_fn: no_etag,
            full_buffering_fn: no_full_buffering,
        }
    }

    /// Configure the buffer size [bytes] used when serving files.
    ///
    /// ## Default
    ///
    /// `64 * 1024` (64 KiB)
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Configure whether to serve dotfiles (e.g., `.env`, `.gitignore`).
    ///
    /// ## Default
    ///
    /// `false`
    pub fn serve_dotfiles(mut self, yes: bool) -> Self {
        self.serve_dotfiles = yes;
        self
    }

    /// Configure a list of file extensions to omit from being served.
    ///
    /// ## Default
    ///
    /// `[]`
    ///
    /// ## Example
    ///
    /// Following example omits `.html` extension from route where each file is served.
    /// e.g.:
    ///
    /// - `./public/abc.html` is served at `/abc`
    /// - `./public/abc.css` is served at `/abc.css`
    ///
    /// Note that this handle `index.html` specially, so that:
    ///
    /// - `./public/index.html` is served at `/`, not at `/index`
    ///
    /// ```rust,no_run
    /// use ohkami::{Ohkami, Route};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new((
    ///         "/".By(ohkami::ServeDir::new("./public")
    ///             .omit_extensions(["html"])
    ///         ),
    ///     )).run("localhost:5000").await
    /// }
    /// ```
    pub fn omit_extensions(mut self, exts: impl IntoIterator<Item = &'static str>) -> Self {
        self.omit_extensions = exts.into_iter().collect();
        self
    }

    /// Configure a function to generate ETag for each file served.
    ///
    /// ## Default
    ///
    /// `None` (no ETag is generated) for any file
    pub fn etag_fn(mut self, func: fn(&Path) -> Option<ETag<'static>>) -> Self {
        self.etag_fn = func;
        self
    }

    /// Configure a function to determine whether to fully buffer the file into memory before serving by each file.
    ///
    /// ## Default
    ///
    /// `false` for any file
    ///
    /// ## Note
    ///
    /// Should return `true` only for **small** and **immutable** files.
    ///
    /// ### Pros
    /// - Reduces disk I/O during serving, potentially improving performance for frequently accessed files.
    /// - Enables faster response times for clients, especially for small to medium-sized files.
    ///
    /// ### Cons
    /// - Increases memory usage, which may be a concern for large files or high traffic scenarios.
    /// - Disables to follow file changes on disk after the initial load.
    pub fn full_buffering_fn(mut self, func: fn(&Path) -> bool) -> Self {
        self.full_buffering_fn = func;
        self
    }
}
