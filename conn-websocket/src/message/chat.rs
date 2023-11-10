use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{
    conn::ConnHandle,
    session::{
        room::RoomHandle,
        session::{Identity, RoomId},
    },
};

use super::protocol::ClientProtocol;

#[derive(Debug, Clone)]
pub enum ConnMessage {
    OnLeave {
        member: Identity,
    },
    OnNewMessage {
        member: Identity,
        message: ClientProtocol,
    },
}

#[derive(Debug, Clone)]
pub enum RoomMessage {
    OnJoin {
        room_id: RoomId,
        member: Identity,
    },
    OnLeave {
        room_id: RoomId,
        member: Identity,
    },
    OnNewMessage {
        room_id: RoomId,
        member: Identity,
        content: Message,
    },
}

pub enum DispatchMessage {
    OnJoin {
        conn_handle: ConnHandle,
    },
    OnNewMessage {
        member: Identity,
        message: ClientProtocol,
    },
    GetMemberCount {
        respond_to: oneshot::Sender<u32>,
    },
}

pub enum SessionMessage {
    OnAccept { conn: ConnHandle },
}
