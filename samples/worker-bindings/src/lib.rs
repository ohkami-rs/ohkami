#[ohkami::bindings]
struct Bindings;

macro_rules! static_assert_eq_str {
    ($left:expr, $right:literal) => {
        const _: [(); true as usize] = [(); 'eq: {
            let (left, right) = ($left.as_bytes(), $right.as_bytes());
            if left.len() != right.len() {
                break 'eq false
            }
            let mut i = 0; while i < left.len() {
                if left[i] != right[i] {
                    break 'eq false
                }
                i += 1;
            }
            true
        } as usize];
    };
}

fn __test_bindings__(bindings: Bindings) {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Bindings>();

    static_assert_eq_str!(Bindings::VARIABLE_1, "hoge");
    static_assert_eq_str!(Bindings::VARIABLE_2, "super fun");

    let _: worker::Ai = bindings.INTELIGENT;

    let _: worker::D1Database = bindings.DB;

    let _: worker::kv::KvStore = bindings.MY_KVSTORE;

    let _: worker::Bucket = bindings.MY_BUCKET;

    let _: worker::Fetcher = bindings.S;

    let _: worker::Queue = bindings.MY_QUEUE;

    let _: worker::ObjectNamespace = bindings.RATE_LIMITER;
}
