pub use ::futures_core::{Stream, ready};


/// # Stream of an async process with a queue
/// 
/// `queue(|mut q| async move { 〜 })` makes an queue for `T` values
/// and an async process that pushes items to the queue, they work as
/// a stream yeilding all the items asynchronously.
/// 
/// <br>
/// 
/// _**note**_ : It's recommended to just `use ohkami::util::stream` and
/// call as **`stream::queue()`**, not direct `queue()`.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::typed::DataStream;
/// use ohkami::util::{StreamExt, stream};
/// use tokio::time::sleep;
/// 
/// #[tokio::main]
/// async fn main() {
///     let qs = stream::queue(|mut q| async move {
///         for i in 1..=5 {
///             sleep(std::time::Duration::from_secs(1)).await;
///             q.push(format!("Hello, I'm message#{i}!"))
///         }
/// 
///         sleep(std::time::Duration::from_secs(1)).await;
/// 
///         q.push("done".to_string())
///     });
/// }
/// ```
/// 
/// <br>
/// 
/// ---
/// *openai.rs*
/// ```ignore
/// use ohkami::prelude::*;
/// use ohkami::Memory;
/// use ohkami::typed::DataStream;
/// use ohkami::util::{StreamExt, stream};
/// 
/// pub async fn relay_chat_completion(
///     api_key: Memory<'_, &'static str>,
///     UserMessage(message): UserMessage,
/// ) -> Result<DataStream<String, Error>, Error> {
///     let mut gpt_response = reqwest::Client::new()
///         .post("https://api.openai.com/v1/chat/completions")
///         .bearer_auth(*api_key)
///         .json(&ChatCompletions {
///             model:    "gpt-4o",
///             stream:   true,
///             messages: vec![
///                 ChatMessage {
///                     role:    Role::user,
///                     content: message,
///                 }
///             ],
///         })
///         .send().await?
///         .bytes_stream();
///     
///     Ok(DataStream::from_stream(stream::queue(|mut q| async move {
///         let mut push_line = |mut line: String| {
///             line.strip_suffix("\n\n").ok();
///     
///             #[cfg(debug_assertions)] {
///                 if line != "[DONE]" {
///                     let chunk: models::ChatCompletionChunk
///                         = serde_json::from_str(&line).unwrap();
///                     let content = chunk
///                         .choices[0]
///                         .delta
///                         .content.as_deref().unwrap_or(""));
///                     print!("{content}");
///                     std::io::Write::flush(&mut std::io::stdout()).ok();
///                 } else {
///                     println!()
///                 }
///             }
///     
///             q.push(Ok(line));
///         };
///     
///         let mut remaining = String::new();
///         while let Some(Ok(raw_chunk)) = gpt_response.next().await {
///             for line in std::str::from_utf8(&raw_chunk).unwrap()
///                 .split_inclusive("\n\n")
///             {
///                 if let Some(data) = line.strip_prefix("data: ") {
///                     if data.ends_with("\n\n") {
///                         push_line(data.to_string())
///                     } else {
///                         remaining = data.into()
///                     }
///                 } else {
///                     #[cfg(debug_assertions)] {
///                         assert!(line.ends_with("\n\n"))
///                     }
///                     push_line(std::mem::take(&mut remaining) + line)
///                 }
///             }
///         }
///     })))
/// }
/// ```
pub fn queue<T, F, Fut>(f: F) -> impls::QueueStream<F, T, Fut>
where
    F:   FnOnce(impls::Queue<T>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    impls::QueueStream::new(f)
}

pub fn once<T>(item: T) -> impls::Once<T> {
    impls::Once(Some(item))
}

pub trait StreamExt: Stream + Sized {
    fn map<T, F: FnMut(Self::Item)->T>(self, f: F) -> impls::Map<Self, F>;
    fn filter<P: FnMut(&Self::Item)->bool>(self, predicate: P) -> impls::Filter<Self, P>;
    fn chain<A: Stream>(self, another: A) -> impls::Chain<Self, A>;
    fn next(&mut self) -> impls::Next<'_, Self>;
}
impl<S: Stream> StreamExt for S {
    #[inline]
    fn map<T, F: FnMut(Self::Item)->T>(self, f: F) -> impls::Map<S, F> {
        impls::Map { inner: self, f }
    }
    fn filter<P: FnMut(&Self::Item)->bool>(self, predicate: P) -> impls::Filter<S, P> {
        impls::Filter { inner: self, predicate }
    }
    fn chain<A: Stream>(self, another: A) -> impls::Chain<Self, A> {
        impls::Chain { inner: self, another }
    }
    fn next(&mut self) -> impls::Next<'_, Self> {
        impls::Next { inner: self }    
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod impls {
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
        #[inline]
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

        impl<T, E> Queue<Result<T, E>> {
            /// `.push(Ok(value))`
            #[inline(always)]
            pub fn add(&mut self, value: T) {
                unsafe {self.0.as_mut()}.push_back(Ok(value))
            }
        }
    };
}
