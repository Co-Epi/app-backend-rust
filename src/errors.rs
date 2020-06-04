use crate::networking::NetworkingError;
use std::{io::ErrorKind, fmt, error, io::Error as StdError};
use tcn::Error as TcnError;
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum ServicesError {
  Networking(NetworkingError),
  Error(Error)
}

impl fmt::Display for ServicesError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{:?}", self)
  }
}

impl From<Error> for ServicesError {
  fn from(error: Error) -> Self {
    ServicesError::Error(error)
  }
}

impl From<NetworkingError> for ServicesError {
  fn from(error: NetworkingError) -> Self {
    ServicesError::Networking(error)
  }
}

impl From<TcnError> for ServicesError {
  fn from(error: TcnError) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, format!("{}", error))))
  }
}

impl From<String> for ServicesError {
  fn from(error: String) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, error)))
  }
}

impl From<&str> for ServicesError {
  fn from(error: &str) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, error)))
  }
}

impl From<serde_json::Error> for ServicesError {
  fn from(error: serde_json::Error) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, format!("{}", error))))
  }
}


impl From<persy::PersyError> for ServicesError {
  fn from(error: persy::PersyError) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, format!("{}", error))))
  }
}


impl From<hex::FromHexError> for ServicesError {
  fn from(error: hex::FromHexError) -> Self {
    ServicesError::Error(Box::new(StdError::new(ErrorKind::Other, format!("{}", error))))
  }
}


impl error::Error for ServicesError {}
