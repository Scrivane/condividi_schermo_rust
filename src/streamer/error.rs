use std::fmt;

//migliorare gestione errori
pub struct ServerError {
    pub(crate) message: String,
}

impl fmt::Debug for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerError: {}", self.message)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerError: {}", self.message)
    }
}

impl std::error::Error for ServerError {}