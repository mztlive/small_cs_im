use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use tokio::sync::mpsc;

use crate::{
    auth::RoomId,
    message::internal::{ConnMessage, DispatchMessage, SessionMessage},
    session::{conn::ConnHandle, room::RoomHandle},
};

use super::collection::Cursor;

const MAX_WAITING_QUEUE_SIZE: usize = 100;

pub struct Manager {
    /// rooms
    rooms: HashMap<RoomId, RoomHandle>,

    /// customer service list
    customer_services: Cursor<ConnHandle>,

    /// waiting queue.  no dispatch conns
    waiting_queue: VecDeque<ConnHandle>,

    /// receive message from session
    mailbox_session: mpsc::Receiver<SessionMessage>,

    /// receive message from conn
    mailbox_conn: mpsc::Receiver<ConnMessage>,
}

impl Manager {
    pub fn new(
        mailbox_session: mpsc::Receiver<SessionMessage>,
        mailbox_conn: mpsc::Receiver<ConnMessage>,
    ) -> Self {
        Manager {
            rooms: HashMap::new(),
            customer_services: Cursor::new(Vec::new()),
            mailbox_session,
            mailbox_conn,
            waiting_queue: VecDeque::new(),
        }
    }

    /// handle received message from session.
    async fn handle_session_message(&mut self, msg: SessionMessage) {
        match msg {
            SessionMessage::OnAccept { conn } => {
                self.add_session(conn).await;
            }
        }
    }

    /// handle received message from conn.
    async fn handle_conn_message(&mut self, msg: ConnMessage) {
        match msg {
            ConnMessage::OnLeave { member } => {
                println!("member leave: {:?}", member);
            }
            ConnMessage::OnNewMessage { member, message } => {
                let room_id = message.room_id();
                if let Some(room_handle) = self.rooms.get(room_id) {
                    let dispatch_message = DispatchMessage::OnNewMessage { member, message };
                    room_handle.new_message(dispatch_message).await;
                }
            }
        }
    }

    /// create room and add conn to room.
    async fn create_room(&mut self, c: ConnHandle, cs: ConnHandle) {
        let room_id = format!("{}-{}", c.identity().id(), cs.identity().id());

        let room_handle = RoomHandle::new(room_id.clone());
        self.rooms.insert(room_id, room_handle.clone());

        room_handle.join(vec![c, cs]).await;
    }

    /// dispatch customer to customer service.
    /// if no customer service online, add customer to waiting queue.
    async fn dispatch(&mut self, customer: ConnHandle) {
        if customer.identity().is_customer_service() {
            return;
        }

        if self.customer_services.is_empty() {
            self.waiting_queue.push_back(customer);
            return;
        }

        let customer_service = self.customer_services.next().unwrap().clone(); // the unwrap forever safe.
        self.create_room(customer, customer_service).await
    }

    // auto dispatch customer to customer service.
    async fn auto_dispatch(&mut self) {
        if self.customer_services.is_empty() {
            return;
        }

        if let Some(cs) = self.waiting_queue.pop_front() {
            self.dispatch(cs).await;
        }
    }

    /// add session. if conn is customer service, add to customer_services.
    /// else dispatch to customer service.
    async fn add_session(&mut self, conn: ConnHandle) {
        if conn.identity().is_customer_service() {
            self.customer_services.push(conn.clone());
            return;
        }

        // try dispatch customer to customer service.
        self.dispatch(conn).await;
    }
}

async fn listener(mut dispatch: Manager) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("auto dispatch");
                dispatch.auto_dispatch().await;
            }

            Some(msg) = dispatch.mailbox_session.recv() => {
                dispatch.handle_session_message(msg).await;
            }
            Some(msg) = dispatch.mailbox_conn.recv() => {
                dispatch.handle_conn_message(msg).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DispatchHandle {
    sender_session: mpsc::Sender<SessionMessage>,
    sender_conn: mpsc::Sender<ConnMessage>,
}

impl DispatchHandle {
    pub fn new() -> Self {
        let (sender_session, mailbox_session) = mpsc::channel(100);
        let (sender_conn, mailbox_conn) = mpsc::channel(100);

        let dispatch = Manager::new(mailbox_session, mailbox_conn);

        tokio::spawn(listener(dispatch));

        DispatchHandle {
            sender_session,
            sender_conn,
        }
    }

    /// Session calls this method to send a message to Dispatch
    pub async fn send_message(&self, message: SessionMessage) {
        if let Err(err) = self.sender_session.send(message).await {
            println!("send message error: {:?}", err);
        }
    }

    /// Conn calls this method to send a message to Dispatch
    pub async fn send_conn_message(&self, message: ConnMessage) {
        let _ = self.sender_conn.send(message).await;
    }
}
