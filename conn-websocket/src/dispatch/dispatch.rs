use std::{collections::HashMap, time::Duration};

use tokio::sync::mpsc;

use crate::{
    conn::ConnHandle,
    message::chat::{ConnMessage, DispatchMessage, SessionMessage},
    session::{room::RoomHandle, session::RoomId},
};

use super::collection::Cursor;

const MAX_WAITING_QUEUE_SIZE: usize = 100;

pub struct Manager {
    /// rooms
    rooms: HashMap<RoomId, RoomHandle>,

    /// customer service list
    customer_services: Cursor<ConnHandle>,

    /// receiver from session
    receiver_from_session: mpsc::Receiver<SessionMessage>,

    /// receiver from conn
    receiver_from_conn: mpsc::Receiver<ConnMessage>,

    /// waiting queue.  store not dispatch customer service of conn.
    waiting_queue: mpsc::Receiver<ConnHandle>,

    /// send_to_waiting_queue
    sender_waiting_queue: mpsc::Sender<ConnHandle>,
}

impl Manager {
    pub fn new(
        receiver_from_session: mpsc::Receiver<SessionMessage>,
        receiver_from_conn: mpsc::Receiver<ConnMessage>,
    ) -> Self {
        let (sender_waiting_queue, waiting_queue) = mpsc::channel(MAX_WAITING_QUEUE_SIZE);

        Manager {
            rooms: HashMap::new(),
            customer_services: Cursor::new(Vec::new()),
            receiver_from_session,
            receiver_from_conn,
            waiting_queue,
            sender_waiting_queue,
        }
    }

    /// handle received message from session.
    async fn handle_message(&mut self, msg: SessionMessage) {
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
                    room_handle.on_new_message(dispatch_message).await;
                }
            }
        }
    }

    /// create room and add conn to room.
    async fn create_room(&mut self, c: ConnHandle, cs: ConnHandle) {
        let room_id = format!("{}-{}", c.identity().identity(), cs.identity().identity());

        let room_handle = RoomHandle::new(room_id.clone());
        self.rooms.insert(room_id, room_handle.clone());

        room_handle.on_join(vec![c, cs]).await;
    }

    /// dispatch customer to customer service.
    /// if no customer service online, add customer to waiting queue.
    async fn dispatch(&mut self, customer: ConnHandle) {
        if customer.identity().is_customer_service() {
            println!(
                "customer service not support dispatch: {:?}",
                customer.identity()
            );
            return;
        }

        if self.customer_services.is_empty() {
            println!("no customer service online. conn prepare to waiting queue.");

            if let Err(err) = self.sender_waiting_queue.send(customer).await {
                println!("send to waiting queue error: {:?}", err);
            }

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

        match self.waiting_queue.try_recv() {
            Ok(customer) => {
                self.dispatch(customer).await;
            }
            Err(err) => println!("try recv waiting queue error: {:?}", err),
        }
    }

    /// add session. if conn is customer service, add to customer_services.
    /// else dispatch to customer service.
    async fn add_session(&mut self, conn: ConnHandle) {
        if conn.identity().is_customer_service() {
            self.customer_services.push(conn.clone());
            return;
        }

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

            Some(msg) = dispatch.receiver_from_session.recv() => {
                dispatch.handle_message(msg).await;
            }
            Some(msg) = dispatch.receiver_from_conn.recv() => {
                dispatch.handle_conn_message(msg).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DispatchHandle {
    sender: mpsc::Sender<SessionMessage>,
    sender_of_conn: mpsc::Sender<ConnMessage>,
}

impl DispatchHandle {
    pub fn new() -> Self {
        // tx is session message sender.
        // rx is session message receiver.
        let (tx, rx) = mpsc::channel(100);

        // conn_tx is conn message sender.
        // conn_rx is conn message receiver.
        let (conn_tx, conn_rx) = mpsc::channel(100);

        let dispatch = Manager::new(rx, conn_rx);

        tokio::spawn(listener(dispatch));

        DispatchHandle {
            sender: tx,
            sender_of_conn: conn_tx,
        }
    }

    /// Session calls this method to send a message to Dispatch
    pub async fn send_message(&self, message: SessionMessage) {
        if let Err(err) = self.sender.send(message).await {
            println!("send message error: {:?}", err);
        }
    }

    /// Conn calls this method to send a message to Dispatch
    pub async fn send_conn_message(&self, message: ConnMessage) {
        let _ = self.sender_of_conn.send(message).await;
    }
}
