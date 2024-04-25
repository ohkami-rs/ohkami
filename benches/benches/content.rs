#![feature(test)]
extern crate test;

use ohkami_lib::{CowSlice, Slice};


struct CowContent(
    CowSlice,
);
impl CowContent {
    #[inline] fn from_request_bytes(bytes: &[u8]) -> Self {
        Self(CowSlice::Ref(Slice::from_bytes(bytes)))
    }
    #[inline] fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(unsafe {self.0.as_bytes()})
    }
}

struct BytesContent(
    ::bytes::Bytes
);
impl BytesContent {
    #[inline] fn from_request_bytes(bytes: &[u8]) -> Self {
        Self(::bytes::Bytes::copy_from_slice(bytes))
    }
    #[inline] fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.0)
    }
}


fn small() -> Vec<u8> {
    Vec::from(test::black_box("\
        Bytes contains a vtable, which allows implementations of Bytes to define \
        how sharing/cloning is implemented in detail. When Bytes::clone() is called, \
        Bytes will call the vtable function for cloning the backing storage in order \
        to share it behind between multiple Bytes instances.\
    "))
}

fn large() -> Vec<u8> {
    Vec::from(test::black_box(r#"\
        [[lib.rs]]

        Provides abstractions for working with bytes.

        The `bytes` crate provides an efficient byte buffer structure
        ([`Bytes`]) and traits for working with buffer
        implementations ([`Buf`], [`BufMut`]).

        # `Bytes`

        `Bytes` is an efficient container for storing and operating on contiguous
        slices of memory. It is intended for use primarily in networking code, but
        could have applications elsewhere as well.

        `Bytes` values facilitate zero-copy network programming by allowing multiple
        `Bytes` objects to point to the same underlying memory. This is managed by
        using a reference count to track when the memory is no longer needed and can
        be freed.

        A `Bytes` handle can be created directly from an existing byte store (such as `&[u8]`
        or `Vec<u8>`), but usually a `BytesMut` is used first and written to. For
        example:
        
        ```rust
        use bytes::{BytesMut, BufMut};
        
        let mut buf = BytesMut::with_capacity(1024);
        buf.put(&b"hello world"[..]);
        buf.put_u16(1234);
        
        let a = buf.split();
        assert_eq!(a, b"hello world\x04\xD2"[..]);
        
        buf.put(&b"goodbye world"[..]);
        
        let b = buf.split();
        assert_eq!(b, b"goodbye world"[..]);
        
        assert_eq!(buf.capacity(), 998);
        ```
        
        In the above example, only a single buffer of 1024 is allocated. The handles
        `a` and `b` will share the underlying buffer and maintain indices tracking
        the view into the buffer represented by the handle.
        
        See the [struct docs](`Bytes`) for more details.
        
        # `Buf`, `BufMut`
        
        These two traits provide read and write access to buffers. The underlying
        storage may or may not be in contiguous memory. For example, `Bytes` is a
        buffer that guarantees contiguous memory, but a [rope] stores the bytes in
        disjoint chunks. `Buf` and `BufMut` maintain cursors tracking the current
        position in the underlying byte storage. When bytes are read or written, the
        cursor is advanced.
        
        [rope]: https://en.wikipedia.org/wiki/Rope_(data_structure)
        
        ## Relation with `Read` and `Write`
        
        At first glance, it may seem that `Buf` and `BufMut` overlap in
        functionality with [`std::io::Read`] and [`std::io::Write`]. However, they
        serve different purposes. A buffer is the value that is provided as an
        argument to `Read::read` and `Write::write`. `Read` and `Write` may then
        perform a syscall, which has the potential of failing. Operations on `Buf`
        and `BufMut` are infallible.

        ---

        [[bytes.rs]]

        A cheaply cloneable and sliceable chunk of contiguous memory.
        
        Bytes is an efficient container for storing and operating on contiguous slices of memory. \
        It is intended for use primarily in networking code, but could have applications elsewhere as well.
        
        Bytes values facilitate zero-copy network programming by allowing multiple Bytes objects \
        to point to the same underlying memory.
        
        Bytes does not have a single implementation. It is an interface, whose exact behavior is implemented \
        through dynamic dispatch in several underlying implementations of Bytes.
        
        All Bytes implementations must fulfill the following requirements:
        
        - They are cheaply cloneable and thereby shareable between an unlimited amount of components, \
        for example by modifying a reference count.
        - Instances can be sliced to refer to a subset of the original buffer.
        
        ## Memory layout
        The Bytes struct itself is fairly small, limited to 4 usize fields used to track information \
        about which segment of the underlying memory the Bytes handle has access to.
        
        Bytes keeps both a pointer to the shared state containing the full memory slice \
        and a pointer to the start of the region visible by the handle. Bytes also tracks the length of its view \
        into the memory.
        
        ## Sharing
        Bytes contains a vtable, which allows implementations of Bytes to define \
        how sharing/cloning is implemented in detail. When Bytes::clone() is called, \
        Bytes will call the vtable function for cloning the backing storage in order \
        to share it behind between multiple Bytes instances.
        
        For Bytes implementations which refer to constant memory (e.g. created via Bytes::from_static()) \
        the cloning implementation will be a no-op.
        
        For Bytes implementations which point to a reference counted shared storage (e.g. an Arc<[u8]>), \
        sharing will be implemented by increasing the reference count.
        
        Due to this mechanism, multiple Bytes instances may point to the same shared memory region. \
        Each Bytes instance can point to different sections within that memory region, and \
        Bytes instances may or may not have overlapping views into the memory.
        
        The following diagram visualizes a scenario where 2 Bytes instances make use of an Arc-based backing storage, \
        and provide access to different views:
        
        ```text
        
           Arc ptrs                   ┌─────────┐
           ________________________ / │ Bytes 2 │
          /                           └─────────┘
         /          ┌───────────┐     |         |
        |_________/ │  Bytes 1  │     |         |
        |           └───────────┘     |         |
        |           |           | ___/ data     | tail
        |      data |      tail |/              |
        v           v           v               v
        ┌─────┬─────┬───────────┬───────────────┬─────┐
        │ Arc │     │           │               │     │
        └─────┴─────┴───────────┴───────────────┴─────┘
        ```
    "#))
}


#[bench] fn create_small_cow(b: &mut test::Bencher) {
    let data = small();

    b.iter(|| {
        let _c = CowContent::from_request_bytes(data.as_slice());
    })
}
#[bench] fn create_large_cow(b: &mut test::Bencher) {
    let data = large();

    b.iter(|| {
        let _c = CowContent::from_request_bytes(data.as_slice());
    })
}

#[bench] fn create_small_bytes(b: &mut test::Bencher) {
    let data = small();

    b.iter(|| {
        let _c = BytesContent::from_request_bytes(data.as_slice());
    })
}
#[bench] fn create_large_bytes(b: &mut test::Bencher) {
    let data = large();

    b.iter(|| {
        let _c = BytesContent::from_request_bytes(data.as_slice());
    })
}


#[bench] fn write_small_cow(b: &mut test::Bencher) {
    let mut buf = Vec::new();

    let data = small();
    let c = CowContent::from_request_bytes(data.as_slice());

    b.iter(|| {
        c.write_to(&mut buf);
    })
}
#[bench] fn write_large_cow(b: &mut test::Bencher) {
    let mut buf = Vec::new();

    let data = large();
    let c = CowContent::from_request_bytes(data.as_slice());

    b.iter(|| {
        c.write_to(&mut buf);
    })
}

#[bench] fn write_small_bytes(b: &mut test::Bencher) {
    let mut buf = Vec::new();

    let data = small();
    let c = BytesContent::from_request_bytes(data.as_slice());

    b.iter(|| {
        c.write_to(&mut buf);
    })
}
#[bench] fn write_large_bytes(b: &mut test::Bencher) {
    let mut buf = Vec::new();

    let data = large();
    let c = BytesContent::from_request_bytes(data.as_slice());

    b.iter(|| {
        c.write_to(&mut buf);
    })
}
