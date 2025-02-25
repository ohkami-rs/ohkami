use ohkami::bindings;
use worker::wasm_bindgen;

#[bindings]
struct AutoBindings;

#[bindings]
struct ManualBindings {
    /* automatically `#[allow(unused)]` */
    VARIABLE_1: bindings::Var,

    #[allow(unused)]
    DB: bindings::D1,

    #[allow(unused)]
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

fn __test_bindings_new__(env: &worker::Env) -> Result<(), worker::Error> {
    let _: AutoBindings = AutoBindings::new(env)?;
    let _: ManualBindings = ManualBindings::new(env)?;
    Ok(())
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn handle_dummy_env() {
    use worker::wasm_bindgen::{JsCast, closure::Closure};
    use worker::js_sys::{Object, Reflect, Function};

    console_error_panic_hook::set_once();

    let dummy_db = {
        let o = Object::new();
        {
            let constructor = Function::unchecked_from_js(Closure::<dyn Fn()>::new(|| {}).into_js_value());
            {
                let attributes = Object::new();
                Reflect::set(&attributes, &"value".into(), &"D1Database".into()).unwrap();
                Reflect::define_property(&constructor, &"name".into(), &attributes).unwrap();
            }
            Reflect::set(&o, &"constructor".into(), &constructor).unwrap();
        }
        o
    };

    let dummy_env = {
        let o = Object::new();
        {
            Reflect::set(&o, &"DB".into(), &dummy_db).unwrap();
            Reflect::set(&o, &"MY_KVSTORE".into(), &Object::new()).unwrap();
        }
        worker::Env::unchecked_from_js(o.unchecked_into())
    };

    let _: ohkami::bindings::D1 = dummy_env.d1("DB").unwrap();

    let _: ohkami::bindings::KV = dummy_env.kv("MY_KVSTORE").unwrap();
}
