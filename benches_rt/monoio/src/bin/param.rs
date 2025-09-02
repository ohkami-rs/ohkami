use ohkami::prelude::*;


#[inline(always)]
async fn echo_id(Path(id): Path<String>) -> String {
    id
}

fn main() {
    monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
        .enable_all()
        .build()
        .unwrap()
        .block_on({
            Ohkami::new((
                "/user/:id"
                .GET(echo_id),
            )).howl("0.0.0.0:3000")
        })
}
