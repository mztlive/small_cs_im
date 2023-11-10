use std::io;

use tokio_tungstenite::tungstenite::Error;

/// returns an io::Error with kind PermissionDenied and message "Invalid token"
pub fn invalid_token_err() -> Error {
    Error::from(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "Invalid token",
    ))
}
