use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    conn,
    message::chat::{DispatchMessage, RoomMessage},
};

use super::session::{Identity, RoomId};

pub struct ChatRoom {
    id: RoomId,

    members: HashMap<Identity, conn::ConnHandle>,

    manager_receiver: mpsc::Receiver<DispatchMessage>,
}

/// ChatRoom is a actor.
impl ChatRoom {
    pub fn new(id: RoomId, receiver: mpsc::Receiver<DispatchMessage>) -> Self {
        ChatRoom {
            id,
            members: HashMap::new(),
            manager_receiver: receiver,
        }
    }

    /// send message to all conn.
    async fn broadcast(&mut self, msg: RoomMessage, filter: Vec<Identity>) {
        for (id, conn_handle) in self.members.iter() {
            if filter.contains(id) {
                continue;
            }

            conn_handle.send_message(msg.clone()).await
        }
    }

    /// send from_member message to all conn. except from_member.
    async fn broadcast_join(&mut self, from_member: Identity) {
        let chat_message = RoomMessage::OnJoin {
            room_id: self.id.clone(),
            member: from_member.clone(),
        };

        self.broadcast(chat_message, vec![from_member]).await;
    }

    /// on message received from dispatch manager or conn.
    /// OnJoin from dispatch manager.
    /// OnLeave from conn.
    /// OnNewMessage from conn.
    async fn handle_dispatch_message(&mut self, msg: DispatchMessage) {
        match msg {
            DispatchMessage::OnJoin { conn_handle } => {
                println!(
                    "member join room: {:?}, member: {:?}",
                    self.id,
                    conn_handle.identity()
                );

                self.members
                    .insert(conn_handle.identity().clone(), conn_handle.clone());

                self.broadcast_join(conn_handle.identity().clone()).await;
            }
            DispatchMessage::OnNewMessage {
                member: from_member,
                message,
            } => {
                let chat_message = RoomMessage::OnNewMessage {
                    room_id: self.id.clone(),
                    member: from_member.clone(),
                    content: message.to_message(),
                };

                self.broadcast(chat_message, vec![from_member]).await;
            }
            DispatchMessage::GetMemberCount { respond_to } => {
                let _ = respond_to.send(self.members.len() as u32);
            }
        }
    }
}

/// listen message from dispatch manager or conn.
async fn listener(mut room: ChatRoom) {
    while let Some(msg) = room.manager_receiver.recv().await {
        room.handle_dispatch_message(msg).await;
    }
}

#[derive(Clone, Debug)]
pub struct RoomHandle {
    id: RoomId,
    sender: mpsc::Sender<DispatchMessage>,
}

impl RoomHandle {
    pub fn new(id: RoomId) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let room = ChatRoom::new(id.clone(), rx);

        tokio::spawn(listener(room));

        RoomHandle { id, sender: tx }
    }

    pub fn id(&self) -> &RoomId {
        &self.id
    }

    /// send message to room
    async fn send_message(&self, message: DispatchMessage) {
        if let Err(err) = self.sender.send(message).await {
            println!("send message error: {:?}", err);
        }
    }

    /// send on join message to room.
    pub async fn on_join(&self, conn_handle: Vec<conn::ConnHandle>) {
        for conn_handle in conn_handle {
            let message = DispatchMessage::OnJoin {
                conn_handle: conn_handle.clone(),
            };
            self.send_message(message).await;
        }
    }

    /// send on new message to room.
    pub async fn on_new_message(&self, message: DispatchMessage) {
        self.send_message(message).await;
    }

    /// return member count in this room.
    pub async fn get_member_count(&self) -> u32 {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let message = DispatchMessage::GetMemberCount { respond_to: tx };

        self.send_message(message).await;

        rx.await.unwrap()
    }
}
