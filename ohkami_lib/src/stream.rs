pub use ::futures_core::{Stream, ready};

use std::task::{Poll, Context};
use std::pin::Pin;
use std::future::Future;


pub fn once<T>(item: T) -> stream::Once<T> {
    Once(Some(item))
}

pub trait StreamExt: Stream + Sized {
    fn map<T, F: FnMut(Self::Item)->T>(self, f: F) -> stream::Map<Self, F>;
    fn filter<P: FnMut(&Self::Item)->bool>(self, predicate: P) -> stream::Filter<Self, P>;
    fn chain<A: Stream>(self, another: A) -> stream::Chain<Self, A>;
    fn next(&mut self) -> stream::Next<'_, Self>;
}
impl<S: Stream> StreamExt for S {
    #[inline]
    fn map<T, F: FnMut(Self::Item)->T>(self, f: F) -> stream::Map<S, F> {
        Map { inner: self, f }
    }
    fn filter<P: FnMut(&Self::Item)->bool>(self, predicate: P) -> stream::Filter<S, P> {
        Filter { inner: self, predicate }
    }
    fn chain<A: Stream>(self, another: A) -> stream::Chain<Self, A> {
        Chain { inner: self, another }
    }
    fn next(&mut self) -> stream::Next<'_, Self> {
        Next { inner: self }    
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////

mod stream {
    pub struct Once<T>(Option<T>);
    impl<T: Unpin> Stream for Once<T> {
        type Item = T;
        #[inline]
        fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(self.get_mut().0.take())
        }
    }

    pub struct Map<S, F> {
        inner: S,
        f:     F,
    }
    impl<S, F, T> Stream for Map<S, F>
    where
        S: Stream,
        F: FnMut(S::Item) -> T,
    {
        type Item = F::Output;
        #[inline]
        fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let res = ready! {
                (unsafe {self.as_mut().map_unchecked_mut(|m| &mut m.inner)})
                .poll_next(cx)
            };
            Poll::Ready(res.map(|item| (unsafe {self.get_unchecked_mut()}.f)(item)))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.inner.size_hint()
        }
    }

    pub struct Filter<S, P> {
        inner:     S,
        predicate: P,
    }
    impl<S, P> Stream for Filter<S, P>
    where
        S: Stream,
        P: FnMut(&S::Item) -> bool,
    {
        type Item = S::Item;
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match ready!(
                (unsafe {self.as_mut().map_unchecked_mut(|m| &mut m.inner)})
                .poll_next(cx)
            ) {
                None => Poll::Ready(None),
                Some(item) => if (unsafe {&mut self.as_mut().get_unchecked_mut().predicate})(&item) {
                    Poll::Ready(Some(item))
                } else {
                    self.poll_next(cx)
                }
            }
        }
    }

    pub struct Chain<S, A> {
        inner:   S,
        another: A,
    }
    impl<S, A, Item> Stream for Chain<S, A>
    where
        S: Stream<Item = Item>,
        A: Stream<Item = Item>,
    {
        type Item = Item;
        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            match ready!((unsafe {self.as_mut().map_unchecked_mut(|this| &mut this.inner)}).poll_next(cx)) {
                Some(item) => Poll::Ready(Some(item)),
                None => (unsafe {self.map_unchecked_mut(|this| &mut this.another)}).poll_next(cx)
            }
        }
    }

    pub struct Next<'n, S> {
        inner: &'n mut S,
    }
    impl<'n, S> Future for Next<'n, S>
    where
        S: Stream,
    {
        type Output = Option<S::Item>;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            (unsafe {self.map_unchecked_mut(|pin| &mut *pin.inner)})
                .poll_next(cx)
        }
    }
}
