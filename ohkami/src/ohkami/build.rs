#![allow(non_snake_case, unused_mut)]

use super::router::TrieRouter;
use crate::handler::{Handlers, ByAnother, Dir};


trait RoutingItem {
    fn apply(self, router: &mut TrieRouter);
} const _: () = {
    impl RoutingItem for Handlers {
        fn apply(self, router: &mut TrieRouter) {
            router.register_handlers(self)
        }
    }

    impl RoutingItem for ByAnother {
        fn apply(self, router: &mut TrieRouter) {
            router.merge_another(self)
        }
    }

    impl RoutingItem for Dir {
        fn apply(self, router: &mut TrieRouter) {
            struct StaticFileHandler {
                mime:     &'static str,
                content:  Vec<u8>,

                /// Used for `Content-Length` header.
                /// 
                /// The size itself can be got by `.content.len()`,
                /// but in response, we have to write it in stringified form
                /// every time. So we should the string here for performance.
                size_str: String,
            } const _: () = {
                impl StaticFileHandler {
                    fn new(path_sections: &[String], file: std::fs::File) -> Result<Self, String> {
                        let filename = path_sections.last()
                            .ok_or_else(|| format!("[.Dir] got empty file path"))?;
                        let (_, extension) = filename.rsplit_once('.')
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: In current version, ohkami doesn't support serving files that have no extenstion"))?;
                        let mime = ohkami_lib::mime::get_by_extension(extension)
                            .ok_or_else(|| format!("[.Dir] got `{filename}`: ohkami doesn't know the extension `{extension}`"))?;

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

                        let size_str = content.len().to_string();

                        Ok(Self { mime, content, size_str })
                    }
                }
                
                impl crate::handler::IntoHandler<std::fs::File> for StaticFileHandler {
                    fn into_handler(self) -> crate::handler::Handler {
                        let this: &'static StaticFileHandler
                            = Box::leak(Box::new(self));

                        crate::handler::Handler::new(|_| Box::pin(async {
                            let mut res = crate::Response::OK();
                            {
                                res.headers.set()
                                    .ContentType(this.mime)
                                    .ContentLength(&*this.size_str);
                                res.content = Some(
                                    std::borrow::Cow::Borrowed(&this.content)
                                );
                            }
                            res
                        }))
                    }
                }
            };

            #[cfg(feature="DEBUG")]
            println!{ "[Dir] .files = {:#?}", self.files }

            for (mut path, file) in self.files {
                let mut handler = match StaticFileHandler::new(&path, file) {
                    Ok(h) => h,
                    Err(msg) => panic!("{msg}")
                };

                if let Some(exts) = self.omit_extensions.as_ref() {
                    if path.last().unwrap() == "index.html" && exts.contains(&"html") {
                        path.pop();
                    } else {
                        for ext in exts.iter() {
                            if let Some(filename) = path.last().unwrap().strip_suffix(&format!(".{ext}")) {
                                let filename_len = filename.len();
                                path.last_mut().unwrap().truncate(filename_len);
                                break
                            }
                        }
                    }
                }

                router.register_handlers(
                    Handlers::new(Box::leak({
                        let base_path = self.route.trim_end_matches('/').to_string();
                        match &*path.join("/") {
                            ""   => base_path,
                            some => base_path + "/" + some,
                        }
                    }.into_boxed_str())).GET(handler)
                );
            }
        }
    }

    /// This is for better developer experience.
    /// 
    /// If we impl `Routes` only for `Handlers` and `ByAnother`, ohkami users
    /// will see following situations：
    /// 
    /// ```ignore
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
    /// ```
    impl RoutingItem for &'static str {
        fn apply(self, _router: &mut TrieRouter) {}
    }
};

pub trait Routes {
    fn apply(self, router: &mut TrieRouter);
} impl<R: RoutingItem> Routes for R {
    fn apply(self, router: &mut TrieRouter) {
        <R as RoutingItem>::apply(self, router)
    }
} macro_rules! impl_for_tuple {
    ( $( $item:ident ),+ ) => {
        impl<$( $item: RoutingItem ),+> Routes for ( $($item,)+ ) {
            fn apply(self, router: &mut TrieRouter) {
                let ( $( $item, )+ ) = self;
                $(
                    <$item as RoutingItem>::apply($item, router);
                )+
            }
        }
    };
} const _: () = {
    impl Routes for () {fn apply(self, _router: &mut TrieRouter) {}}
    
    impl_for_tuple!(R1);
    impl_for_tuple!(R1, R2);
    impl_for_tuple!(R1, R2, R3);
    impl_for_tuple!(R1, R2, R3, R4);
    impl_for_tuple!(R1, R2, R3, R4, R5);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15);
    impl_for_tuple!(R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16);
};
