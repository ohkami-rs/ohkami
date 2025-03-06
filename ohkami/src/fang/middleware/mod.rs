pub mod util;
use super::{Fang, BoxedFPC, SendSyncOnNative};


#[allow(private_interfaces)]
pub trait Fangs: SendSyncOnNative + 'static {
    // returning box for object-safety
    fn build(&self, inner: BoxedFPC) -> BoxedFPC;

    #[cfg(feature="openapi")]
    fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation;
}

#[allow(private_interfaces)]
const _: () = {
    impl<F: Fang<BoxedFPC>> Fangs for F {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            BoxedFPC::from_proc(self.chain(inner))
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            <Self as Fang<BoxedFPC>>::openapi_map_operation(self, operation)
        }
    }

    impl Fangs for () {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            inner
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            operation
        }
    }

    impl<
        F1: Fang<BoxedFPC>,
    > Fangs for (F1,)
    {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1,) = self;
            BoxedFPC::from_proc(
                f1.chain(inner)
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1,) = self;
            f1.openapi_map_operation(operation)
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<BoxedFPC>,
    > Fangs for (F1, F2) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(inner)
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(operation)
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(inner)
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(operation)
                )
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3, F4) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3, f4) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(inner)
                        )
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3, f4) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(
                        f4.openapi_map_operation(operation)
                    )
                )
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3, F4, F5) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3, f4, f5) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(inner)
                            )
                        )
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3, f4, f5) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(
                        f4.openapi_map_operation(
                            f5.openapi_map_operation(operation)
                        )
                    )
                )
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3, F4, F5, F6) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(inner)
                                )
                            )
                        )
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3, f4, f5, f6) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(
                        f4.openapi_map_operation(
                            f5.openapi_map_operation(
                                f6.openapi_map_operation(operation)
                            )
                        )
                    )
                )
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<F7::Proc>,
        F7: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3, F4, F5, F6, F7) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6, f7) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(
                                        f7.chain(inner)
                                    )
                                )
                            )
                        )
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3, f4, f5, f6, f7) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(
                        f4.openapi_map_operation(
                            f5.openapi_map_operation(
                                f6.openapi_map_operation(
                                    f7.openapi_map_operation(operation)
                                )
                            )
                        )
                    )
                )
            )
        }
    }

    impl<
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<F7::Proc>,
        F7: Fang<F8::Proc>,
        F8: Fang<BoxedFPC>,
    > Fangs for (F1, F2, F3, F4, F5, F6, F7, F8) {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6, f7, f8) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(
                                        f7.chain(
                                            f8.chain(inner)
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: crate::openapi::Operation) -> crate::openapi::Operation {
            let (f1, f2, f3, f4, f5, f6, f7, f8) = self;
            f1.openapi_map_operation(
                f2.openapi_map_operation(
                    f3.openapi_map_operation(
                        f4.openapi_map_operation(
                            f5.openapi_map_operation(
                                f6.openapi_map_operation(
                                    f7.openapi_map_operation(
                                        f8.openapi_map_operation(operation)
                                    )
                                )
                            )
                        )
                    )
                )
            )
        }
    }
};
