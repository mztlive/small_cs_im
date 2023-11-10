use jsonwebtoken::{DecodingKey, Validation};
use tokio_tungstenite::{
    accept_hdr_async,
    tungstenite::handshake::server::{ErrorResponse, Request, Response},
    WebSocketStream,
};

use tokio::net::TcpStream;

use super::{Member, UserType};

#[derive(Debug)]
pub struct ConnWrapper {
    pub stream: WebSocketStream<TcpStream>,
    pub member: Member,
}

pub async fn handshake(stream: TcpStream) -> Result<ConnWrapper, ErrorResponse> {
    let mut user_id = String::new();
    let mut user_type = UserType::Customer;
    let mut user_name = String::new();

    let callback = |request: &Request, response: Response| {
        let headers = request.headers();

        let auth_header = match headers.get("Authorization") {
            Some(auth_header) => auth_header,
            None => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        let token = match auth_header.to_str() {
            Ok(token_str) => {
                if !token_str.starts_with("Bearer ") {
                    return Err(ErrorResponse::new(Some("invalid token".to_string())));
                }

                token_str[7..].to_string()
            }
            Err(_) => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        let secret = DecodingKey::from_secret(b"aoquoquoeq");
        let decode_res =
            jsonwebtoken::decode::<Member>(token.as_str(), &secret, &Validation::default());

        match decode_res {
            Ok(token_data) => {
                user_id = token_data.claims.identity().to_string();
                user_type = token_data.claims.user_type();
                user_name = token_data.claims.user_name().to_string();
            }
            Err(_) => return Err(ErrorResponse::new(Some("invalid token".to_string()))),
        };

        Ok(response)
    };

    let ws_stream = accept_hdr_async(stream, callback)
        .await
        .map_err(|_| ErrorResponse::new(Some("WebSocket handshake failed".to_string())))?;

    Ok(ConnWrapper {
        stream: ws_stream,
        member: Member::new(user_type, user_id, user_name),
    })
}
