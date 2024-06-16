pub use ::futures_core::{Stream, ready};


pub fn queue<T, F, Fut>(f: F) -> stream::QueueStream<F, T, Fut>
where
    F:   FnOnce(stream::Queue<T>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    stream::QueueStream::new(f)
}

pub fn once<T>(item: T) -> stream::Once<T> {
    stream::Once(Some(item))
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
        stream::Map { inner: self, f }
    }
    fn filter<P: FnMut(&Self::Item)->bool>(self, predicate: P) -> stream::Filter<S, P> {
        stream::Filter { inner: self, predicate }
    }
    fn chain<A: Stream>(self, another: A) -> stream::Chain<Self, A> {
        stream::Chain { inner: self, another }
    }
    fn next(&mut self) -> stream::Next<'_, Self> {
        stream::Next { inner: self }    
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////

mod stream {
    use super::{Stream, ready};
    use std::task::{Poll, Context};
    use std::pin::Pin;
    use std::future::Future;


    pub struct Once<T>(
        pub(super) Option<T>
    );
    impl<T: Unpin> Stream for Once<T> {
        type Item = T;
        #[inline]
        fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(self.get_mut().0.take())
        }
    }

    pub struct Map<S, F> {
        pub(super) inner: S,
        pub(super) f:     F,
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
        pub(super) inner:     S,
        pub(super) predicate: P,
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
        pub(super) inner:   S,
        pub(super) another: A,
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
        pub(super) inner: &'n mut S,
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

    pub struct QueueStream<F, T, Fut> {
        queue: std::collections::VecDeque<T>,
        proc:  QueuingingProc<F, T>,
        queuing_future: Option<Fut>,
        queuing_state:  Option<std::ptr::NonNull<Fut>>,
    }
    struct QueuingingProc<F, T> {
        queue_ptr: Option<std::ptr::NonNull<std::collections::VecDeque<T>>>,
        /// `Option<_>` to `take` in `setup`
        f: Option<F>,
    }
    pub struct Queue<T>(
        std::ptr::NonNull<std::collections::VecDeque<T>>
    );
    const _: () = {
        unsafe impl<F:Send, T:Send, Fut:Send> Send for QueueStream<F, T, Fut> {}
        unsafe impl<F:Send, T: Send> Send for QueuingingProc<F, T> {}
        unsafe impl<T:Send> Send for Queue<T> {}

        use std::collections::VecDeque;
        use std::ptr::NonNull;
        
        impl<F, T, Fut> QueueStream<F, T, Fut>
        where
            F:   FnOnce(Queue<T>) -> Fut,
            Fut: Future<Output = ()>,
        {
            pub fn new(f: F) -> Self {
                Self {
                    queue: VecDeque::new(),
                    proc:  QueuingingProc { f: Some(f), queue_ptr: None },
                    queuing_future: None,
                    queuing_state:  None
                }
            }

            #[inline]
            fn setup(self: Pin<&mut Self>) {
                if self.proc.queue_ptr.is_none() {
                    let this = unsafe {self.get_unchecked_mut()};
                    this.proc.queue_ptr = Some(unsafe {NonNull::new_unchecked(
                        &mut this.queue
                    )});

                    let user_queue = Queue(this.proc.queue_ptr.unwrap());
                    this.queuing_future = Some((this.proc.f.take().unwrap())(user_queue));
                    this.queuing_state  = Some(unsafe {NonNull::new_unchecked(
                        this.queuing_future.as_mut().unwrap_unchecked()
                    )});
                }
            }

            #[inline]
            fn poll_queuing_future(&mut self, cx: &mut Context<'_>) -> Poll<()> {
                if self.queuing_state.is_none() {
                    Poll::Ready(())
                } else {
                    let poll = (unsafe {Pin::new_unchecked(self.queuing_state.as_mut().unwrap_unchecked().as_mut())}).poll(cx);
                    if poll.is_ready() {
                        self.queuing_state = None
                    }
                    poll
                }
            }
        }

        impl<F, T, Fut> Stream for QueueStream<F, T, Fut>
        where
            F:   FnOnce(Queue<T>) -> Fut,
            Fut: Future<Output = ()>,
        {
            type Item = T;
            fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                self.as_mut().setup();

                let this = unsafe {self.get_unchecked_mut()};

                match this.poll_queuing_future(cx) {
                    Poll::Ready(()) => match this.queue.pop_front() {
                        None        => Poll::Ready(None),
                        Some(value) => Poll::Ready(Some(value)),
                    }
                    Poll::Pending => match this.queue.pop_front() {
                        None        => Poll::Pending,
                        Some(value) => Poll::Ready(Some(value)),
                    }
                }
            }
        }

        impl<T> Queue<T> {
            #[inline(always)]
            pub fn push(&mut self, value: T) {
                unsafe {self.0.as_mut()}.push_back(value)
            }
        }
    };
}
