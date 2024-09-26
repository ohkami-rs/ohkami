use std::env;
use ohkami::util::{StreamExt, stream};
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

    /* Handle reqwest's annoying buffering */
    let mut chat_completion_chunk_stream = stream::queue(|mut q| async move {
        let mut push_line = |mut line: String| {
            #[cfg(debug_assertions)] {
                assert!(line.ends_with("\n\n"))
            }
            line.truncate(line.len() - 2);
            q.push(line)
        };

        let mut remaining = String::new();

        while let Some(Ok(raw_chunk)) = gpt_response.next().await {
            for line in std::str::from_utf8(&raw_chunk).unwrap()
                .split_inclusive("\n\n")
            {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.ends_with("\n\n") {
                        push_line(data.to_string())
                    } else {
                        remaining = data.into()
                    }
                } else {
                    push_line(std::mem::take(&mut remaining) + line)
                }
            }
        }
    });

    while let Some(chunk) = chat_completion_chunk_stream.next().await {
        println!("\n\n[chunk]\n---------------------------\n{}\n---------------------------\n",
            chunk
                .replace('\n', r"\n")
                .replace('\r', r"\r")
        );
    }
}
