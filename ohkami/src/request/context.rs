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
    worker: WorkerContext,

    #[cfg(feature="rt_lambda")]
    lambda: LambdaContext,
}

#[cfg(feature="rt_worker")]
struct WorkerContext {
    ctx: ::worker::Context,
    env: ::worker::Env,
}

#[cfg(feature="rt_lambda")]
enum LambdaContext {
    HTTP(::crate::x_lambda::LambdaHTTPRequestContext),

    #[cfg(feature="ws")]
    WebSocket(::crate::x_lambda::LambdaWebSocketRequestContext),
}

impl Context {
    #[cfg(feature="__rt__")]
    pub(super) const fn init() -> Self {
        Self {
            store: None,

            #[cfg(feature="rt_worker")]
            worker: ,
        }
    }

    #[allow(unused)]
    pub(crate) fn clear(&mut self) {
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
}
