use ohkami::prelude::*;


#[inline(always)]
async fn echo_id(id: String) -> String {
    id
}

fn main() {
    smol::block_on({
        Ohkami::new((
            "/user/:id"
                .GET(echo_id),
        )).howl("0.0.0.0:3000")
    })
}
