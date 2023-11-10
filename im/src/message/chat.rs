use tokio::sync::oneshot;
use tungstenite::Message;

use crate::{
    auth::{Member, RoomId},
    session::conn::ConnHandle,
};

use super::protocol::ClientProtocol;

#[derive(Debug, Clone)]
pub enum ConnMessage {
    OnLeave {
        member: Member,
    },
    OnNewMessage {
        member: Member,
        message: ClientProtocol,
    },
}

#[derive(Debug, Clone)]
pub enum RoomMessage {
    OnJoin {
        room_id: RoomId,
        member: Member,
    },
    OnLeave {
        room_id: RoomId,
        member: Member,
    },
    OnNewMessage {
        room_id: RoomId,
        member: Member,
        content: Message,
    },
}

pub enum DispatchMessage {
    OnJoin {
        conn_handle: ConnHandle,
    },
    OnNewMessage {
        member: Member,
        message: ClientProtocol,
    },
    GetMemberCount {
        respond_to: oneshot::Sender<u32>,
    },
}

pub enum SessionMessage {
    OnAccept { conn: ConnHandle },
}
