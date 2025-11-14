use ohkami::prelude::*;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .event_interval(11)
        .global_queue_interval(31)
        .build()
        .expect("Failed building the Runtime")
        .block_on(Ohkami::new((
            "/user/:id"
                .GET(|Path(id): Path<String>| async {id}),
        )).run("0.0.0.0:3000"))
}
