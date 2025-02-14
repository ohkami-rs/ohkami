use super::{Handler, IntoHandler};
use super::super::{Fang, BoxedFPC, middleware::Fangs};


impl<H: IntoHandler<T>, T, F1> IntoHandler<(F1, H, T)> for (F1, H)
where
    F1: Fang<BoxedFPC>
{
    fn n_params(&self) -> usize {self.1.n_params()}

    fn into_handler(self) -> Handler {
        let (f, h) = self;
        let h = h.into_handler();
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature="openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation)
        }
    }
}

impl<H: IntoHandler<T>, T, F1, F2> IntoHandler<(F1, F2, H, T)> for (F1, F2, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<BoxedFPC>,
{
    fn n_params(&self) -> usize {self.2.n_params()}

    fn into_handler(self) -> Handler {
        let (f1, f2, h) = self;
        let h = h.into_handler();
        let f = (f1, f2);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature="openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation)
        }
    }
}

impl<H: IntoHandler<T>, T, F1, F2, F3> IntoHandler<(F1, F2, F3, H, T)> for (F1, F2, F3, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<F3::Proc>,
    F3: Fang<BoxedFPC>,
{
    fn n_params(&self) -> usize {self.3.n_params()}

    fn into_handler(self) -> Handler {
        let (f1, f2, f3, h) = self;
        let h = h.into_handler();
        let f = (f1, f2, f3);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature="openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation)
        }
    }
}

impl<H: IntoHandler<T>, T, F1, F2, F3, F4> IntoHandler<(F1, F2, F3, F4, H, T)> for (F1, F2, F3, F4, H)
where
    F1: Fang<F2::Proc>,
    F2: Fang<F3::Proc>,
    F3: Fang<F4::Proc>,
    F4: Fang<BoxedFPC>,
{
    fn n_params(&self) -> usize {self.4.n_params()}

    fn into_handler(self) -> Handler {
        let (f1, f2, f3, f4, h) = self;
        let h = h.into_handler();
        let f = (f1, f2, f3, f4);
        Handler {
            proc: Fangs::build(&f, h.proc),
            #[cfg(feature="openapi")]
            openapi_operation: Fangs::openapi_map_operation(&f, h.openapi_operation)
        }
    }
}
