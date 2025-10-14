use ohkami::prelude::*;
use glommio::{LocalExecutorPoolBuilder, PoolPlacement, CpuSet, executor};

async fn echo_id(Path(id): Path<String>) -> String {
    let executor = executor();
    executor.spawn_blocking(move || id).await
}

fn main() {
    LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
        dbg!(std::thread::available_parallelism().unwrap_or(1).get()),
        dbg!(CpuSet::online().ok())
    )).on_all_shards(|| {
        Ohkami::new((
            "/user/:id"
                .GET(echo_id),
        )).howl("0.0.0.0:3000")
    }).unwrap().join_all();
}
