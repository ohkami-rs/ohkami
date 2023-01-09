use std::{io::{BufReader, Read}, fs::File};
use ohkami::{prelude::*, components::json::Json};
use once_cell::sync::Lazy;

#[derive(JSON)]
struct Dinosaur {
    name:        String,
    description: String,
}

static DATA_STR: Lazy<String> = Lazy::new(|| {
    let mut buffer = BufReader::new(
        File::open("./data.json").expect("failed to open file")
    );
    let mut data = String::new();
    buffer.read_to_string(&mut data).expect("failed to read data from buffer");
    data
});
static DATA: Lazy<Vec<Dinosaur>> = Lazy::new(|| {
    let mut raw = <Vec<Dinosaur> as Json>::de(&DATA_STR)
        .expect("failed to deserilize data");
    for data in &mut raw {
        (data.name).make_ascii_lowercase() // convert to lower case in advance
    }
    raw
});

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", || async {Response::OK("Welcome to dinosaur API!")})
        .GET("/api", || async {Response::OK(DATA_STR.as_str())})
        .GET("/api/:dinosaur", get_one_by_name)
        .howl(":8080")
}

async fn get_one_by_name(c: Context, name: String) -> Result<Response> {
    let index = DATA
        .binary_search_by_key(&name.to_ascii_lowercase().as_str(), |data| &data.name)
        ._else(|_| Response::BadRequest("No dinosaurs found"))?;
    c.OK(&DATA[index])
}
