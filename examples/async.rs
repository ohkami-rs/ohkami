#![feature(trait_alias)]
use std::collections::HashMap;
use futures::Future;


fn main() -> Result<(), String> {
    let m = Map::new();
    m.resister(handler);
    let res = async_std::task::block_on(m.run(1))?;
    println!("{res}");
    Ok(())
}

trait Handler<F> = Fn() -> F
where F: Future<Output = usize>;

struct Map<'h>(
    usize,
    HashMap<
        usize,
        &'h dyn Fn() -> dyn Future<Output = usize>
    >
);impl<'h> Map<'h> {
    fn new() -> Self {
        Self(0, HashMap::new())
    }
    fn resister<F>(&mut self, f: dyn Handler<F>) {
        self.0 += 1;
        self.1.insert(self.0, f);
    }
    async fn run(&self, id: usize) -> usize {// Result<usize, String> {
        let handler = self.1.get(&id).unwrap();
        // Ok(handler().await)
        handler().await
    }
}

async fn handler() -> usize { 2 }