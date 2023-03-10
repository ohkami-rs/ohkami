use super::Fang;

pub trait IntoFang {
    fn into_fang(self) -> Fang;
}
