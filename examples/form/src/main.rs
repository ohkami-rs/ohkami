use ohkami::utils::{Payload, File};

#[Payload(FormData)]
struct SubmitData1 {
    account_name: String,
}

#[Payload(FormData)]
struct SubmitData2 {
    account_name: String,
    pics:         Vec<File>,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
