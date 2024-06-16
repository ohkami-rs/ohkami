pub mod error;
pub mod fangs;
pub mod models;

use error::Error;
use models::{UserMessage, ChatMessage, ChatCompletions, Role};

use ohkami::prelude::*;
use ohkami::Memory;
use ohkami::typed::DataStream;
use ohkami::utils::StreamExt;


#[tokio::main]
async fn main() {
    Ohkami::with((
        fangs::WithAPIKey::from_env().expect("\
            OpenAI API key is not found. \n\
            \n\
            [USAGE]\n\
            Run `cargo run` with one of \n\
              a. Set an environment variable `OPENAI_API_KEY` to your API key\n\
              b. Pass your API key by command line arguments `-- --api-key ＜here＞`\n\
        "),
    ), (
        "/chat-once".POST(relay_chat_completion),
    )).howl("localhost:5050").await
}

pub async fn relay_chat_completion(
    api_key: Memory<'_, &'static str>,
    UserMessage(message): UserMessage,
) -> Result<DataStream<String, Error>, Error> {
    let gpt_response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(*api_key)
        .json(&ChatCompletions {
            model:    "gpt-4o",
            stream:   true,
            messages: vec![
                ChatMessage {
                    role:    Role::user,
                    content: message,
                }
            ],
        })
        .send().await?
        .bytes_stream();

    Ok(DataStream::from_stream(gpt_response.map(|chunk| {
        let mut chunk = &*chunk?;
        if chunk.starts_with(b"data: ") {
            chunk = &chunk[b"data: ".len()..]
        }
        while chunk.ends_with(b"\n") {
            chunk = &chunk[..chunk.len()-1]
        }

        println!("\n\n[chunk]\n---------------------------\n{}\n---------------------------\n",
            std::str::from_utf8(chunk).unwrap()
                .replace('\n', r"\n")
                .replace('\r', r"\r")
        );

        Ok(String::from_utf8(chunk.into()).unwrap())
    })))
}
