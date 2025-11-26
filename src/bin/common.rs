use chrono::Local;
use serde::{
    Serialize,
    Deserialize,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    UserMessage,
    SystemMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    // pub id: u128,
    pub username: String,
    pub content: String,
    pub timestamp: String,
    pub message_type: MessageType,
}

impl ChatMessage {
    pub fn json(&self) -> Result<String, serde_json::Error> {
        return serde_json::to_string(&self);
    }
}

pub fn create_msg(username: String, content: String, message_type: MessageType) -> ChatMessage {
    let time = Local::now().format("%H:%M:%S");

    match message_type {
        MessageType::UserMessage => {
            return ChatMessage {
                // id,
                username,
                content,
                timestamp: time.to_string(),
                message_type,
            }
        }

        MessageType::SystemMessage => {
            return ChatMessage {
                username: "System".to_string(),
                content,
                timestamp: time.to_string(),
                message_type,
            }
        }
    }
}

