use ohkami::prelude::*;


#[inline(always)]
async fn echo_id(Path(id): Path<String>) -> String {
    id
}

fn main() {
    compio::runtime::Runtime::new().unwrap().block_on({
        Ohkami::new((
            "/user/:id"
            .GET(echo_id),
        )).run("0.0.0.0:3000")
    })
}
