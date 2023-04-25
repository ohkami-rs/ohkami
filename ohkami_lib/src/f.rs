#[macro_export]
macro_rules! f {
    ($string:literal $(, $arg:expr)*) => {
        format!($string $(, $arg)*)
    };
    ({ $( $content:tt )+ }) => {
        ohkami::__::json!({ $( $content )+ })
    };
}
