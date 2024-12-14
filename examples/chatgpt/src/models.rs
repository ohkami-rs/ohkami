use ohkami::serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ChatCompletions {
    pub model:    &'static str,
    pub messages: Vec<ChatMessage>,
    pub stream:   bool,
}
#[derive(Serialize)]
pub struct ChatMessage {
    pub role:    Role,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatCompletionChunk {
    pub id:      String,
    pub choices: [ChatCompletionChoice; 1],
}
#[derive(Deserialize)]
pub struct ChatCompletionChoice {
    pub delta:         ChatCompletionDelta,
    pub finish_reason: Option<ChatCompletionFinishReason>,
}
#[derive(Deserialize)]
pub struct ChatCompletionDelta {
    pub role:    Option<Role>,
    pub content: Option<String>,
}
#[derive(Deserialize)]
#[allow(non_camel_case_types)]
pub enum ChatCompletionFinishReason {
    stop,
    length,
    content_filter,
}

#[derive(Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum Role {
    system,
    user,
    assistant,
}
