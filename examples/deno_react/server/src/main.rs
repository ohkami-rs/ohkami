use std::{io::{BufReader, Read}, fs::File};
use serde::{Serialize, Deserialize};
use ohkami::prelude::*;
use once_cell::sync::Lazy;

#[derive(Serialize, Deserialize)]
struct Dinosaur {
    name:        String,
    sexcription: String,
}

static DATA_STR: Lazy<String> = Lazy::new(|| {
    let mut buffer = BufReader::new(
        File::open("./data.json").expect("failed to open file")
    );
    let mut data = String::new();
    buffer.read_to_string(&mut data).expect("failed to read data from buffer");
    data
});
static DATA: Lazy<Vec<Dinosaur>> = Lazy::new(||
    json(DATA_STR.as_str()).unwrap()
        .to_struct::<Vec<Dinosaur>>()
        .expect("failed to deserilize data")
);

fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK("Welcome to dinosaur API!")})
        .GET("/api", |_| async {Response::OK(DATA_STR.as_str())})
        .GET("/api/:dinosaur", get_one_by_name)
        .serve_on(":8000")
}

async fn get_one_by_name(ctx: Context) -> Result<Response> {
    let name = ctx.param().unwrap()
        .to_ascii_lowercase();
    let index = DATA
        .binary_search_by_key(&name.as_str(), |data| &data.name)
        ._else(|_| Response::BadRequest("No dinosaurs found"))?;
    Response::OK(json(&DATA[index])?)
}
