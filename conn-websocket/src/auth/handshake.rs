use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::handshake::server::{Callback, ErrorResponse, Request, Response},
    WebSocketStream,
};

use tokio::net::TcpStream;

use crate::session::session::UserType;

#[derive(Debug)]
pub struct ConnWrapper {
    pub stream: WebSocketStream<TcpStream>,
    pub user_type: UserType,
    pub identity: String,
}

pub async fn handle(stream: TcpStream) -> Result<ConnWrapper, ErrorResponse> {
    let mut user_type = UserType::Customer;
    let mut token = String::new();

    let callback = |request: &Request, response: Response| {
        let headers = request.headers();

        let auth_header = match headers.get("Authorization") {
            Some(auth_header) => auth_header,
            None => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        match auth_header.to_str() {
            Ok(token_str) => {
                if !token_str.starts_with("Bearer ") {
                    return Err(ErrorResponse::new(Some("invalid token".to_string())));
                }

                token = token_str[7..].to_string();
            }
            Err(_) => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        }

        let user_type_header = match headers.get("User-Type") {
            Some(user_type_header) => user_type_header,
            None => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        user_type = match user_type_header.to_str() {
            Ok(user_type_str) => match user_type_str {
                "customer" => UserType::Customer,
                "customer_service" => UserType::CustomerService,
                _ => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
            },
            Err(_) => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback)
        .await
        .map_err(|_| ErrorResponse::new(Some("WebSocket handshake failed".to_string())))?;

    Ok(ConnWrapper {
        stream: ws_stream,
        user_type,
        identity: token,
    })
}
