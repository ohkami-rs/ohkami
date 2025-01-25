pub mod error;
pub mod fangs;
pub mod models;

use fangs::APIKey;
use error::Error;
use models::{ChatMessage, ChatCompletions, Role};

use ohkami::prelude::*;
use ohkami::format::Text;
use ohkami::sse::DataStream;
use ohkami::util::StreamExt;

#[tokio::main]
async fn main() {
    let api_key = APIKey::from_env();

    println!("Try:\n\
        curl -v 'http://localhost:5050/chat-once' -H 'Content-Type: text/plain' -d '＜your question＞'\n\
    ");

    Ohkami::new((
        api_key,
        "/chat-once"
            .POST(relay_chat_completion),
    )).howl("localhost:5050").await
}

pub async fn relay_chat_completion(
    Context(APIKey(api_key)): Context<'_, APIKey>,
    Text(content): Text<String>,
) -> Result<DataStream, Error> {
    let mut gpt_response = reqwest::Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&ChatCompletions {
            model:    "gpt-4o",
            stream:   true,
            messages: vec![
                ChatMessage {
                    role: Role::user,
                    content,
                }
            ],
        })
        .send().await?
        .bytes_stream();

    Ok(DataStream::new(|mut s| async move {
        let mut send_line = |mut line: String| {
            #[cfg(debug_assertions)] {
                assert!(line.ends_with("\n\n"))
            }
            if line.ends_with("\n\n") {
                line.truncate(line.len() - 2);
            }

            #[cfg(debug_assertions)] {
                if line != "[DONE]" {
                    use ohkami::serde::json;

                    let chunk: models::ChatCompletionChunk = json::from_slice(line.as_bytes()).unwrap();
                    print!("{}", chunk.choices[0].delta.content.as_deref().unwrap_or(""));
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                } else {
                    println!()
                }
            }

            s.send(line);
        };

        let mut remaining = String::new();
        while let Some(Ok(raw_chunk)) = gpt_response.next().await {
            for line in std::str::from_utf8(&raw_chunk).unwrap()
                .split_inclusive("\n\n")
            {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data.ends_with("\n\n") {
                        send_line(data.to_string())
                    } else {
                        remaining = data.into()
                    }
                } else {
                    #[cfg(debug_assertions)] {
                        assert!(line.ends_with("\n\n"))
                    }
                    send_line(std::mem::take(&mut remaining) + line)
                }
            }
        }
    }))
}
