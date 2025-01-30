use ohkami_lib::map::TupleMap;
use std::any::{Any, TypeId};

pub struct Context {
    store: Option<Box<
        TupleMap<
            TypeId,
            Box<dyn Any + Send + Sync>
        >
    >>,

    #[cfg(feature="rt_worker")]
    worker: std::mem::MaybeUninit<(::worker::Context, ::worker::Env)>,

    #[cfg(feature="rt_lambda")]
    lambda: Option<Box<crate::x_lambda::LambdaHTTPRequestContext>>,
}

impl Context {
    #[cfg(feature="__rt__")]
    pub(super) const fn init() -> Self {
        Self {
            store: None,

            #[cfg(feature="rt_worker")]
            worker: None,

            #[cfg(feature="rt_lambda")]
            lambda: None,
        }
    }

    #[cfg(feature="rt_worker")]
    pub(super) fn load(&mut self, worker: (::worker::Context, ::worker::Env)) {
        self.worker.write(worker);
    }

    #[cfg(feature="rt_lambda")]
    pub(super) fn load(&mut self, request_context: crate::x_lambda::LambdaHTTPRequestContext) {
        self.lambda = Some(Box::new(request_context));
    }

    #[cfg(feature="__rt_native__")]
    pub(super) fn clear(&mut self) {
        if let Some(map) = &mut self.store {
            map.clear()
        }
    }
}

impl Context {
    #[inline]
    pub fn set<Data: Send + Sync + 'static>(&mut self, value: Data) {
        if self.store.is_none() {
            self.store = Some(Box::new(TupleMap::new()));
        }
        (unsafe {self.store.as_mut().unwrap_unchecked()})
            .insert(TypeId::of::<Data>(), Box::new(value));
    }

    #[inline]
    pub fn get<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.store.as_ref().and_then(|map| map
            .get(&TypeId::of::<Data>())
            .map(|boxed| {
                let data: &dyn Any = &**boxed;
                #[cfg(debug_assertions)] {
                    assert!(data.is::<Data>(), "Request store is poisoned!!!");
                }
                unsafe { &*(data as *const dyn Any as *const Data) }
            })
        )
    }

    #[cfg(feature="rt_worker")]
    #[inline(always)]
    /// SAFETY: MUST be called after `load`
    pub(super) unsafe fn worker(&self) -> &::worker::Env {
        self.worker.assume_init_ref()
    }

    #[cfg(feature="rt_lambda")]
    #[inline(always)]
    /// SAFETY: MUST be called after `load`
    pub(super) unsafe fn lambda(&self) -> &crate::x_lambda::LambdaHTTPRequestContext {
        self.lambda.as_ref().unwrap_unchecked()
    }
}
