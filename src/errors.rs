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

impl error::Error for ServicesError {}
