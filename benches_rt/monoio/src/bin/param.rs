use ohkami::prelude::*;

fn main() {
    let ncpus = std::thread::available_parallelism().map_or(1, |x| x.get());
    
    let runtime = || monoio::RuntimeBuilder::<monoio::IoUringDriver>::new()
        .enable_all()
        .build()
        .unwrap();

    for core in 1..dbg!(ncpus) {
        std::thread::spawn(move || {
            monoio::utils::bind_to_cpu_set([core]).unwrap();            
            runtime().block_on({
                Ohkami::new((
                    "/user/:id"
                        .GET(async |Path(id): Path<String>| id),
                )).run("0.0.0.0:3000")
            });
        });
    }
    
    monoio::utils::bind_to_cpu_set([0]).unwrap();            
    runtime().block_on({
        Ohkami::new((
            "/user/:id"
                .GET(async |Path(id): Path<String>| id),
        )).run("0.0.0.0:3000")
    });
}
