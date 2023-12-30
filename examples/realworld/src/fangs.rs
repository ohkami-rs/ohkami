use ohkami::{Fang, IntoFang, Context, Request};


pub struct Auth;
impl IntoFang for Auth {
    fn into_fang(self) -> Fang {
        Fang(|c: &mut Context, req: &mut Request| {
            todo!()
        })
    }
}
