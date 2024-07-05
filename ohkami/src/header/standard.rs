pub(crate) struct Standard<const N: usize, Value> {
    index:  [u8; N],
    values: Vec<Value>,
}

impl<const N: usize, Value> Standard<N, Value> {
    const NULL: u8 = u8::MAX;

    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            index:  [Self::NULL; N],
            values: Vec::with_capacity(N / 4)
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn get(&self, index: usize) -> Option<&Value> {
        match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(self.values.get_unchecked(index as usize))
        }
    }
    #[inline(always)]
    pub(crate) unsafe fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        match *self.index.get_unchecked(index) {
            Self::NULL => None,
            index      => Some(self.values.get_unchecked_mut(index as usize))
        }
    }

    #[inline(always)]
    pub(crate) unsafe fn delete(&mut self, index: usize) {
        *self.index.get_unchecked_mut(index) = Self::NULL
    }

    #[inline(always)]
    pub(crate) unsafe fn set(&mut self, index: usize, value: Value) {
        *self.index.get_unchecked_mut(index) = self.values.len() as u8;
        self.values.push(value);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &Value)> {
        self.index.iter()
            .enumerate()
            .filter(|(_, index)| **index != Self::NULL)
            .map(|(h, index)| unsafe {(
                h, self.values.get_unchecked(*index as usize)
            )})
    }
}

const _: () = {
    impl<const N: usize, Value: PartialEq> PartialEq for Standard<N, Value> {
        fn eq(&self, other: &Self) -> bool {
            for i in 0..N {
                if unsafe {self.get(i)} != unsafe {other.get(i)} {
                    return false
                }
            }; true
        }
    }

    impl<const N: usize, Value: Clone> Clone for Standard<N, Value> {
        fn clone(&self) -> Self {
            Self {
                index:  self.index.clone(),
                values: self.values.clone(),
            }
        }
    }
};
