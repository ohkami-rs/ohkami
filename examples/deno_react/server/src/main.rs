use std::{io::{BufReader, Read}, fs::File};
use serde::{Serialize, Deserialize};
use ohkami::prelude::*;

#[derive(Serialize, Deserialize)]
struct Dinosaur {
    name:        String,
    sexcription: String,
}

fn main() -> Result<()> {
    let data = JSON({
        let mut data = String::new();
        let buffer = BufReader::new(File::open("./data.json")?);
        buffer.read_to_string(&mut data)?;
        data
    })?;

    println!("Hello, world!");
}
