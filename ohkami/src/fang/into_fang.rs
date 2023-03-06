use super::Fang;

pub trait IntoFang<'req> {
    fn into_fang(self) -> (FangRoute, Fang<'req>);
}
