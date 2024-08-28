use ohkami::prelude::*;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .event_interval(1)
        .build()
        .expect("Failed building the Runtime")
        .block_on(Ohkami::new((
            "/user/:id"
                .GET(|id: String| async {id}),
        )).howl("0.0.0.0:3000"))
}
