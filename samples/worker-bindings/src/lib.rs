use ohkami::bindings;

#[bindings]
struct AutoBindings;

#[bindings]
struct ManualBindings {
    VARIABLE_1: bindings::Var,
    DB: bindings::D1,
    MY_KVSTORE: bindings::KV,
}

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

fn __test_auto_bindings__(bindings: AutoBindings) {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AutoBindings>();

    static_assert_eq_str!(AutoBindings::VARIABLE_1, "hoge");
    static_assert_eq_str!(AutoBindings::VARIABLE_2, "super fun");

    let _: worker::Ai = bindings.INTELIGENT;

    let _: worker::D1Database = bindings.DB;

    let _: worker::kv::KvStore = bindings.MY_KVSTORE;

    let _: worker::Bucket = bindings.MY_BUCKET;

    let _: worker::Fetcher = bindings.S;

    let _: worker::Queue = bindings.MY_QUEUE;

    let _: worker::ObjectNamespace = bindings.RATE_LIMITER;
}

fn __test_manual_bindings__(bindings: ManualBindings) {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ManualBindings>();

    static_assert_eq_str!(ManualBindings::VARIABLE_1, "hoge");

    let _: worker::D1Database = bindings.DB;

    let _: worker::kv::KvStore = bindings.MY_KVSTORE;
}
