use anyhow::Result;
use serde::{Deserialize, Serialize};
use tungstenite::Message;

use crate::session::session::{Identity, RoomId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Tips,
    Chat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientProtocol {
    body: String,
    msg_type: MessageType,
    room_id: RoomId,
}

impl ClientProtocol {
    pub fn new_tips(body: String, room_id: RoomId) -> Self {
        ClientProtocol {
            body,
            msg_type: MessageType::Tips,
            room_id,
        }
    }

    pub fn join(id: Identity, room_id: RoomId) -> Self {
        let msg = format!("{} 加入了聊天", id.identity());
        ClientProtocol::new_tips(msg, room_id)
    }

    pub fn self_join(room_id: RoomId) -> Self {
        let msg = format!("你加入了聊天");
        ClientProtocol::new_tips(msg, room_id)
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn room_id(&self) -> &RoomId {
        &self.room_id
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn msg_type(&self) -> &MessageType {
        &self.msg_type
    }

    pub fn to_message(&self) -> Message {
        // this is a bug, if you use this function, you should check the return value.
        Message::Text(self.to_json().unwrap())
    }
}
