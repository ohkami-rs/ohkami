use std::env;
use ohkami::utils::StreamExt;
use openai::models::{ChatCompletions, ChatMessage, Role};


#[tokio::main]
async fn main() {
    let mut gpt_response = reqwest::Client::builder()
        .build().unwrap()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(env::var("OPENAI_API_KEY").expect("env var `OPENAI_API_KEY` is required"))
        .json(&ChatCompletions {
            model:    "gpt-4o",
            stream:   true,
            messages: vec![
                ChatMessage {
                    role:    Role::user,
                    content: env::args().nth(1).expect("CLI arg (message) is required"),
                }
            ],
        })
        .send().await.expect("reqwest failed")
        .bytes_stream();

    let;

    while let Some(Ok(chunk)) = gpt_response.next().await {
        println!("\n\n[chunk]\n---------------------------\n{}\n---------------------------\n",
            std::str::from_utf8(&chunk).unwrap()
                .replace('\n', r"\n")
                .replace('\r', r"\r")
        );
    }
}
