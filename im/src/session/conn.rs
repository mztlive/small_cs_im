use futures_util::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};

use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{
    auth::Member,
    dispatch::DispatchHandle,
    message::{
        internal::{ConnMessage, RoomMessage},
        protocol::{self, ClientProtocol},
    },
};

/// wrapper websocket connection (actor)
pub struct Conn {
    id: Member,

    /// write is a websocket stream. it can send message to client.
    write: SplitSink<WebSocketStream<TcpStream>, Message>,

    /// read is a websocket stream. it can receive message from client.
    read: SplitStream<WebSocketStream<TcpStream>>,

    /// receiver_from_room is a channel. it can receive message from room.
    mailbox: mpsc::Receiver<RoomMessage>,

    /// dispatch actor handle. use this to send message to dispatch.
    dispatch_handle: DispatchHandle,
}

impl Conn {
    pub fn new(id: Member, stream: WebSocketStream<TcpStream>, mailbox: mpsc::Receiver<RoomMessage>, dispatch_handle: DispatchHandle) -> Self {
        let (write, read) = stream.split();

        Conn {
            id,
            write,
            read,
            mailbox,
            dispatch_handle,
        }
    }

    /// on message received from room.
    /// froward these message to client.
    async fn handle_room_message(&mut self, message: RoomMessage) {
        match message {
            RoomMessage::OnJoin { room_id, member } => {
                if member == self.id {
                    let _ = self.write.send(protocol::self_join(room_id).to_message()).await;

                    return;
                }

                let _ = self.write.send(protocol::join(member, room_id).to_message()).await;
            }
            RoomMessage::OnLeave { room_id, member } => todo!(),
            RoomMessage::OnNewMessage { room_id, member, content } => {
                if member == self.id {
                    return;
                }

                if let Err(err) = self.write.send(content).await {
                    println!("send message to client err: {:?}", err);
                }
            }
        }
    }

    /// on message received from client.
    /// forward these message to dispatc.
    async fn handle_client_message(&mut self, message: Message) {
        match message {
            Message::Text(msg) => {
                let msg: ClientProtocol = match serde_json::from_str(&msg) {
                    Ok(ret) => ret,
                    Err(err) => {
                        println!("parse message error: {:?}", err);
                        return;
                    }
                };

                self.dispatch_handle
                    .send_conn_message(ConnMessage::OnNewMessage {
                        member: self.id.clone(),
                        message: msg,
                    })
                    .await;
            }
            Message::Close(_) => {
                let room_msg = ConnMessage::OnLeave { member: self.id.clone() };

                self.dispatch_handle.send_conn_message(room_msg).await
            }
            _ => todo!(),
        }
    }
}

/// listener for conn actor.
async fn listener(mut conn: Conn) {
    loop {
        tokio::select! {

            // receive message from room.
            Some(msg) = conn.mailbox.recv() => {
                conn.handle_room_message(msg).await;
            }

            // receive message from client.
            Some(msg) = conn.read.next() => {
                match msg {
                    Ok(msg) => {
                        conn.handle_client_message(msg).await;
                    }
                    Err(err) => {
                        println!("receive message from client err: {:?}", err);
                    }
                }
            }
        }
    }
}

/// conn actor handle. use this to send message to conn.
#[derive(Debug, Clone)]
pub struct ConnHandle {
    id: Member,
    tx: mpsc::Sender<RoomMessage>,
}

impl ConnHandle {
    pub fn new(id: Member, stream: WebSocketStream<TcpStream>, dispatch_handle: DispatchHandle) -> Self {
        let (tx, rx) = mpsc::channel(100);

        let conn = Conn::new(id.clone(), stream, rx, dispatch_handle);

        tokio::spawn(listener(conn));

        ConnHandle { id, tx }
    }

    pub fn identity(&self) -> &Member {
        &self.id
    }

    pub async fn send_message(&self, message: RoomMessage) {
        if let Err(err) = self.tx.send(message).await {
            println!("send message error: {:?}", err);
        }
    }
}
