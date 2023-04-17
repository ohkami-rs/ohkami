#[macro_export]
macro_rules! f {
    ($string:literal $(, $arg:expr)*) => {
        format!($string $(, $arg)*)
    };
    ({ $( $key:literal : $value:expr ),+ }) => {
        ohkami::macros::json!({ $( $key : $value ),+ })
    };
}
