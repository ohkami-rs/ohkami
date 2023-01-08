pub trait Serialize {
    fn serialize(&self) -> String;
}

impl Serialize for &str {
    fn serialize(&self) -> String {
        format!(r#""{self}""#)
    }
}
impl Serialize for String {
    fn serialize(&self) -> String {
        format!(r#""{self}""#)
    }
}

impl Serialize for bool {
    fn serialize(&self) -> String {
        String::from(if *self {"true"} else {"false"})
    }
}

macro_rules! impl_for_int {
    ($( $int:ty )*) => {
        $(
            impl Serialize for $int {
                fn serialize(&self) -> String {
                    self.to_string()
                }
            }
        )*
    };
} impl_for_int!(
    u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize
);

impl<S: Serialize> Serialize for Vec<S> {
    fn serialize(&self) -> String {
        let mut s = self.into_iter().fold(
            String::from("["),
            |it, next| it + &next.serialize() + ","
        );
        s.pop(); s + "]"
    }
}
impl<S: Serialize, const N: usize> Serialize for [S; N] {
    fn serialize(&self) -> String {
        let mut s = self.into_iter().fold(
            String::from("["),
            |it, next| it + &next.serialize() + ","
        );
        s.pop(); s + "]"
    }
}