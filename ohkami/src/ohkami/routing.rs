#![allow(non_snake_case, unused_mut)]

use crate::router::{base::Router, segments::RouteSegments};
use crate::fang::{Fang, BoxedFPC};
use crate::fang::handler::{Handler, IntoHandler};
use crate::response::Content;
use crate::Ohkami;
use std::sync::Arc;


#[derive(Clone)]
pub(crate) struct HandlerMeta {
    pub(crate) name: &'static str,
    pub(crate) n_params: usize,
}
impl HandlerMeta {
    fn new<T, H: IntoHandler<T>>(h: &H) -> Self {
        Self {
            name: std::any::type_name::<H>(),
            n_params: h.n_params(),
        }
    }
}
impl std::fmt::Debug for HandlerMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("HandlerMeta")
            .field("name", &self.name)
            .field("n_params", &self.n_params)
            .finish()
    }
}

macro_rules! HandlerSet {
    ($( $method:ident ),*) => {
        pub struct HandlerSet {
            pub(crate) route: RouteSegments,
            $(
                pub(crate) $method: Option<(Handler, HandlerMeta)>,
            )*
        }
        
        impl HandlerSet {
            pub(crate) fn new(route_str: &'static str) -> Self {
                Self {
                    route: RouteSegments::from_literal(route_str),
                    $(
                        $method: None,
                    )*
                }
            }
        }

        impl HandlerSet {
            $(
                pub fn $method<T, H: IntoHandler<T>>(mut self, handler: H) -> Self {
                    let meta = HandlerMeta::new::<T, H>(&handler);
                    self.$method = Some((handler.into_handler(), meta));
                    self
                }
            )*
        }
    };
} HandlerSet! { GET, PUT, POST, PATCH, DELETE }

pub struct ByAnother {
    pub(crate) route:  RouteSegments,
    pub(crate) ohkami: Ohkami,
}

pub struct Dir {
    pub(crate) route: &'static str,
    pub(crate) files: Vec<(
        Vec<String>,
        std::fs::File,
    )>,

    /*=== config ===*/

    /// File extensions (leading `.` trimmed) that should not be appeared in handling path
    pub(crate) omit_extensions: Option<Box<[&'static str]>>,
}
impl Dir {
    fn new(route: &'static str, dir_path: std::path::PathBuf) -> std::io::Result<Self> {
        let dir_path = dir_path.canonicalize()?;

        if !dir_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{} is not directory", dir_path.display()))
            )
        }

        let mut files = Vec::new(); {
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
                    let path_Segments = entry.canonicalize()?
                        .components()
                        .skip(dir_path.components().count())
                        .map(|c| c.as_os_str().to_os_string()
                            .into_string()
                            .map_err(|os_string| std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Can't read a path segment `{}`", os_string.as_encoded_bytes().escape_ascii())
                            ))
                        )
                        .collect::<std::io::Result<Vec<_>>>()?;

                    if path_Segments.last().unwrap().starts_with('.') {
                        crate::warning!("\
                            =========\n\
                            [WARNING] `Route::Dir`: found `{}` in directory `{}`, \
                            are you sure to serve this file？\n\
                            =========\n",
                            entry.display(),
                            dir_path.display(),
                        )
                    }

                    files.push((
                        path_Segments,
                        std::fs::File::open(entry)?
                    ));

                } else if entry.is_dir() {
                    entries.append(&mut fetch_entries(entry)?)

                } else {
                    continue
                }
            }
        }

        Ok(Self {
            route,
            files,

            omit_extensions: None,
        })
    }

    pub fn omit_extensions<const N: usize>(mut self, target_extensions: [&'static str; N]) -> Self {
        self.omit_extensions = Some(Box::new(
            target_extensions.map(|ext| ext.trim_start_matches('.'))
        ));
        self
    }
}


macro_rules! Route {
    ($( $method:ident ),*) => {
        /// Core trait for Ohkami's routing definition.
        /// 
        /// <br>
        /// 
        /// *example.rs*
        /// ```no_run
        /// use ohkami::{Ohkami, Route};
        /// 
        /// async fn index() -> &'static str {
        ///     "ohkami"
        /// }
        /// 
        /// async fn greet() -> &'static str {
        ///     "I'm fine."
        /// }
        /// 
        /// async fn hello() -> String {
        ///     format!("Hello!!!")
        /// }
        /// 
        /// #[tokio::main]
        /// async fn main() {
        ///     Ohkami::new((
        ///         "/"  // <-- `Route` works here...
        ///             .GET(index),
        ///         "/hello"  // <-- `Route` works here...
        ///             .GET(greet)
        ///             .PUT(hello),
        ///     )).howl("localhost:3000").await
        /// }
        /// ```
        pub trait Route {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> HandlerSet;
            )*

            fn By(self, another: Ohkami) -> ByAnother;

            fn Dir(self, static_files_dir_path: &'static str) -> Dir;
        }

        impl Route for &'static str {
            $(
                fn $method<T>(self, handler: impl IntoHandler<T>) -> HandlerSet {
                    HandlerSet::new(self).$method(handler)
                }
            )*

            fn By(self, another: Ohkami) -> ByAnother {
                ByAnother {
                    route:  RouteSegments::from_literal(self),
                    ohkami: another,
                }
            }

            fn Dir(self, path: &'static str) -> Dir {
                // Check `self` is valid route
                let _ = RouteSegments::from_literal(self);

                match Dir::new(
                    self,
                    path.into()
                ) {
                    Ok(dir) => dir,
                    Err(e) => panic!("{e}")
                }
            }
        }
    };
} Route! { GET, PUT, POST, PATCH, DELETE }


trait RoutingItem {
    fn apply(self, router: &mut Router);
}
const _: () = {
    impl RoutingItem for HandlerSet {
        fn apply(self, router: &mut Router) {
            router.register_handlers(self)
        }
    }

    impl RoutingItem for ByAnother {
        fn apply(self, router: &mut Router) {
            router.merge_another(self)
        }
    }

    impl RoutingItem for Ohkami {
        fn apply(self, router: &mut Router) {
            router.merge_another(ByAnother {
                route:  RouteSegments::from_literal("/"),
                ohkami: self,
            })
        }
    }

    impl RoutingItem for Dir {
        fn apply(self, router: &mut Router) {
            #[derive(Clone)]
            struct StaticFileHandler {
                mime:     &'static str,
                content:  std::sync::Arc<Vec<u8>>,
            }
            const _: () = {
                impl StaticFileHandler {
                    fn new(path_Segments: &[String], file: std::fs::File) -> Result<Self, String> {
                        let filename = path_Segments.last()
                            .ok_or_else(|| format!("[.Dir] got empty file path"))?;
                        let (_, extension) = filename.rsplit_once('.')
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: In current version, ohkami doesn't support serving files that have no extenstion"))?;
                        let mime = ohkami_lib::mime::get_by_extension(extension)
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: ohkami doesn't know extension `{extension}`"))?;

                        let mut content = vec![
                            u8::default();
                            file.metadata().unwrap().len() as usize
                        ]; {use std::io::Read;
                            let mut file = file;
                            file.read_exact(&mut content)
                                .map_err(|e| e.to_string())?;
                        }

                        if mime.starts_with("text/")
                        && std::str::from_utf8(&content).is_err() {
                            return Err(format!("[.Dir] got `{filename}`: Ohkami doesn't support non UTF-8 text file"))
                        }

                        Ok(Self { mime, content:std::sync::Arc::new(content) })
                    }
                }
                
                impl IntoHandler<std::fs::File> for StaticFileHandler {
                    fn n_params(&self) -> usize {0}
                
                    fn into_handler(self) -> Handler {
                        let this: &'static StaticFileHandler
                            = Box::leak(Box::new(self));

                        Handler::new(|_| Box::pin(async {
                            let mut res = crate::Response::OK();
                            {
                                res.headers.set().ContentType(this.mime);
                                res.content = Content::Payload({
                                    let content: &'static [u8] = &this.content;
                                    content.into()
                                });
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
            };

            #[cfg(feature="DEBUG")]
            println!{ "[Dir] .files = {:#?}", self.files }

            let mut register = |path: Vec<String>, handler: StaticFileHandler| router.register_handlers(
                HandlerSet::new(Box::leak({
                    let base_path = self.route.trim_end_matches('/').to_string();
                    match &*path.join("/") {
                        ""   => if !base_path.is_empty() {base_path} else {"/".into()},
                        some => base_path + "/" + some,
                    }
                }.into_boxed_str())).GET(handler)
            );

            for (mut path, file) in self.files {
                let mut handler = match StaticFileHandler::new(&path, file) {
                    Ok(h) => h,
                    Err(msg) => panic!("{msg}")
                };

                if matches!(&**path.last().unwrap(), "index.html") {
                    if !(self.omit_extensions.as_ref().is_some_and(|exts| exts.contains(&"html"))) {
                        register(path.clone(), handler.clone());
                    }

                    path.pop();
                }

                if let Some(exts) = self.omit_extensions.as_ref() {
                    for ext in exts.iter() {
                        if let Some(filename) = path.last().and_then(|p| p.strip_suffix(&format!(".{ext}"))) {
                            let filename_len = filename.len();
                            path.last_mut().unwrap().truncate(filename_len);
                            break
                        }
                    }
                }

                register(path, handler);
            }
        }
    }

    /// This is for better developer experience.
    /// 
    /// If we don't impl `Routes` `&str`, ohkami users
    /// will see following situations：
    /// 
    /// ---
    /// fn my_ohkami() -> Ohkami {
    ///     Ohkami::new((
    ///         "/".|
    /// /*          ↑ cursor */
    ///     ))
    /// }
    /// 
    /// // Here rust-analyzer puts red underlines for all lines of `Ohkami::new(( 〜 ))`
    /// // because the type of argument of `new` is `&str` **AT NOW** and `Routes` trait is
    /// // NOT IMPLEMENTED for this.
    /// // 
    /// // This must be so annoying!!!
    /// ---
    impl RoutingItem for &'static str {
        fn apply(self, _router: &mut Router) {}
    }
};

pub trait Routing<Fangs = ()> {
    fn apply(self, target: &mut Ohkami);
}
const _: () = {
    impl Routing<()> for () {
        fn apply(self, _target: &mut Ohkami) {}
    }
    impl<R: RoutingItem> Routing<()> for R {
        fn apply(self, target: &mut Ohkami) {
            <R as RoutingItem>::apply(self, &mut target.router)
        }
    }

    /// for better developer experience
    impl<F: Fang<BoxedFPC> + 'static> Routing<std::marker::PhantomData<F>> for F {
        fn apply(self, target: &mut Ohkami) {
            target.fangs = Some(Arc::new(self));
        }
    }
    impl<F: Fang<BoxedFPC> + 'static> Routing<(std::marker::PhantomData<F>,)> for (F,) {
        fn apply(self, target: &mut Ohkami) {
            let (f,) = self;
            target.fangs = Some(Arc::new(f));
        }
    }

    macro_rules! routing {
        ( $( $item:ident ),+ ) => {
            impl<$( $item: RoutingItem ),+> Routing<()> for ( $($item,)+ ) {
                fn apply(self, target: &mut Ohkami) {
                    let ( $( $item, )+ ) = self;
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing!(R1);
    routing!(R1, R2);
    routing!(R1, R2, R3);
    routing!(R1, R2, R3, R4);
    routing!(R1, R2, R3, R4, R5);
    routing!(R1, R2, R3, R4, R5, R6);
    routing!(R1, R2, R3, R4, R5, R6, R7);
    routing!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_1_fang {
        ( $( $item:ident ),+ ) => {
            impl<F1, $( $item: RoutingItem ),+> Routing<(F1,)> for ( F1, $($item,)+ )
            where
                F1: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new(f1));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_1_fang!(R1);
    routing_with_1_fang!(R1, R2);
    routing_with_1_fang!(R1, R2, R3);
    routing_with_1_fang!(R1, R2, R3, R4);
    routing_with_1_fang!(R1, R2, R3, R4, R5);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_1_fang!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_2_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, $( $item: RoutingItem ),+> Routing<(F1, F2)> for ( F1, F2, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_2_fangs!(R1);
    routing_with_2_fangs!(R1, R2);
    routing_with_2_fangs!(R1, R2, R3);
    routing_with_2_fangs!(R1, R2, R3, R4);
    routing_with_2_fangs!(R1, R2, R3, R4, R5);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_2_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_3_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, $( $item: RoutingItem ),+> Routing<(F1, F2, F3)> for ( F1, F2, F3, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_3_fangs!(R1);
    routing_with_3_fangs!(R1, R2);
    routing_with_3_fangs!(R1, R2, R3);
    routing_with_3_fangs!(R1, R2, R3, R4);
    routing_with_3_fangs!(R1, R2, R3, R4, R5);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_3_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_4_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, F4, $( $item: RoutingItem ),+> Routing<(F1, F2, F3, F4)> for ( F1, F2, F3, F4, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<F4::Proc> + 'static,
                F4: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, f4, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3, f4)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_4_fangs!(R1);
    routing_with_4_fangs!(R1, R2);
    routing_with_4_fangs!(R1, R2, R3);
    routing_with_4_fangs!(R1, R2, R3, R4);
    routing_with_4_fangs!(R1, R2, R3, R4, R5);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_4_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_5_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, F4, F5, $( $item: RoutingItem ),+> Routing<(F1, F2, F3, F4, F5)> for ( F1, F2, F3, F4, F5, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<F4::Proc> + 'static,
                F4: Fang<F5::Proc> + 'static,
                F5: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, f4, f5, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3, f4, f5)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_5_fangs!(R1);
    routing_with_5_fangs!(R1, R2);
    routing_with_5_fangs!(R1, R2, R3);
    routing_with_5_fangs!(R1, R2, R3, R4);
    routing_with_5_fangs!(R1, R2, R3, R4, R5);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_5_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_6_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, F4, F5, F6, $( $item: RoutingItem ),+> Routing<(F1, F2, F3, F4, F5, F6)> for ( F1, F2, F3, F4, F5, F6, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<F4::Proc> + 'static,
                F4: Fang<F5::Proc> + 'static,
                F5: Fang<F6::Proc> + 'static,
                F6: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, f4, f5, f6, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3, f4, f5, f6)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_6_fangs!(R1);
    routing_with_6_fangs!(R1, R2);
    routing_with_6_fangs!(R1, R2, R3);
    routing_with_6_fangs!(R1, R2, R3, R4);
    routing_with_6_fangs!(R1, R2, R3, R4, R5);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_6_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_7_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, F4, F5, F6, F7, $( $item: RoutingItem ),+> Routing<(F1, F2, F3, F4, F5, F6, F7)> for ( F1, F2, F3, F4, F5, F6, F7, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<F4::Proc> + 'static,
                F4: Fang<F5::Proc> + 'static,
                F5: Fang<F6::Proc> + 'static,
                F6: Fang<F7::Proc> + 'static,
                F7: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, f4, f5, f6, f7, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3, f4, f5, f6, f7)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_7_fangs!(R1);
    routing_with_7_fangs!(R1, R2);
    routing_with_7_fangs!(R1, R2, R3);
    routing_with_7_fangs!(R1, R2, R3, R4);
    routing_with_7_fangs!(R1, R2, R3, R4, R5);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_7_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);

    macro_rules! routing_with_8_fangs {
        ( $( $item:ident ),+ ) => {
            impl<F1, F2, F3, F4, F5, F6, F7, F8, $( $item: RoutingItem ),+> Routing<(F1, F2, F3, F4, F5, F6, F7, F8)> for ( F1, F2, F3, F4, F5, F6, F7, F8, $($item,)+ )
            where
                F1: Fang<F2::Proc> + 'static,
                F2: Fang<F3::Proc> + 'static,
                F3: Fang<F4::Proc> + 'static,
                F4: Fang<F5::Proc> + 'static,
                F5: Fang<F6::Proc> + 'static,
                F6: Fang<F7::Proc> + 'static,
                F7: Fang<F8::Proc> + 'static,
                F8: Fang<BoxedFPC> + 'static,
            {
                fn apply(self, target: &mut Ohkami) {
                    let ( f1, f2, f3, f4, f5, f6, f7, f8, $( $item, )+ ) = self;
                    target.fangs = Some(Arc::new((f1, f2, f3, f4, f5, f6, f7, f8)));
                    $(
                        <$item as RoutingItem>::apply($item, &mut target.router);
                    )+
                }
            }
        };
    }
    routing_with_8_fangs!(R1);
    routing_with_8_fangs!(R1, R2);
    routing_with_8_fangs!(R1, R2, R3);
    routing_with_8_fangs!(R1, R2, R3, R4);
    routing_with_8_fangs!(R1, R2, R3, R4, R5);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7, R8);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    routing_with_8_fangs!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);
};
