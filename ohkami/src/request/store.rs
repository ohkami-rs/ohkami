use ohkami_lib::map::TupleMap;
use std::any::{Any, TypeId};

pub struct Store(
    Option<Box<
        TupleMap<
            TypeId,
            Box<dyn Any + Send + Sync>
        >
    >>
);

impl Store {
    #[cfg(feature="__rt__")]
    pub(super) const fn init() -> Self {
        Self(None)
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        if let Some(map) = &mut self.0 {
            map.clear()
        }
    }

    #[inline]
    pub fn insert<Data: Send + Sync + 'static>(&mut self, value: Data) {
        if self.0.is_none() {
            self.0 = Some(Box::new(TupleMap::new()));
        }
        unsafe {self.0.as_mut().unwrap_unchecked()}
            .insert(TypeId::of::<Data>(), Box::new(value));
    }

    #[inline]
    pub fn get<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.0.as_ref().and_then(|map| map
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
