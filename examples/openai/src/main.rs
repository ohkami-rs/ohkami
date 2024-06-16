pub mod error;
pub mod fangs;
pub mod models;

use error::Error;
use models::{UserMessage, ChatMessage, ChatCompletions, Role};

use ohkami::prelude::*;
use ohkami::Memory;
use ohkami::typed::DataStream;
use ohkami::utils::{StreamExt, stream};


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
    let mut gpt_response = reqwest::Client::new()
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

    Ok(DataStream::from_stream(stream::queue(|mut q| async move {
        let mut push_line = |mut line: String| {
            #[cfg(debug_assertions)] {
                assert!(line.ends_with("\n\n"))
            }

            line.truncate(line.len() - 2);

            #[cfg(debug_assertions)] {
                if line != "[DONE]" {
                    use ohkami::{typed::PayloadType, builtin::payload::JSON};

                    let chunk: models::ChatCompletionChunk = JSON::parse(line.as_bytes()).unwrap();
                    print!("{}", chunk.choices[0].delta.content.as_deref().unwrap_or(""));
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                } else {
                    println!()
                }
            }

            q.push(Ok(line));
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
                    #[cfg(debug_assertions)] {
                        assert!(line.ends_with("\n\n"))
                    }
                    push_line(std::mem::take(&mut remaining) + line)
                }
            }
        }
    })))
}
