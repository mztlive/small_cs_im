pub mod auth;
pub mod dispatch;
pub mod message;
pub mod session;

use std::sync::Arc;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

use crate::{message::chat::SessionMessage, session::conn::ConnHandle};

#[tokio::main]
async fn main() {
    let socket = TcpListener::bind("0.0.0.0:9001").await;
    let listener = socket.expect("failed to bind");

    println!("Listening on: {}", listener.local_addr().unwrap());

    let dispatch_handle = dispatch::DispatchHandle::new();
    while let Ok((stream, _)) = listener.accept().await {
        let handle = dispatch_handle.clone();

        tokio::spawn(async move {
            let conn_wrapper = match auth::handshake::handle(stream).await {
                Ok(conn_wrapper) => conn_wrapper,
                Err(err) => {
                    println!("handshake error: {:?}", err);
                    return;
                }
            };

            let conn_handle =
                ConnHandle::new(conn_wrapper.member, conn_wrapper.stream, handle.clone());

            let message = SessionMessage::OnAccept { conn: conn_handle };

            let _ = handle.clone().send_message(message).await;
        });
    }
}
