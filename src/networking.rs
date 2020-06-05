use reqwest::{blocking::{Client, Response}, Error};
use core::fmt;
use std::error;

static BASE_URL: &str = "https://zmqh8rwdx4.execute-api.us-west-2.amazonaws.com/v4/tcnreport/0.4.0";
// static BASE_URL: &str = "https://v1.api.coepi.org/tcnreport/v0.4.0";

static UNKNOWN_HTTP_STATUS: u16 = 520;

pub trait TcnApi {
  fn get_reports(&self, interval_number: u64, interval_length: u64) -> Result<Vec<String>, NetworkingError>;
  fn post_report(&self, report: String) -> Result<(), NetworkingError>;
}

pub struct TcnApiMock {}

impl TcnApi for TcnApiMock {
  fn get_reports(&self, _interval_number: u64, _interval_length: u64) -> Result<Vec<String>, NetworkingError> {
    Err(NetworkingError{ http_status: 500, message: "Not impl".to_string()})
  }

  fn post_report(&self, _report: String) -> Result<(), NetworkingError> {
    Ok(())
  }
}

pub struct TcnApiImpl {}

impl TcnApiImpl {
  fn create_client() -> Result<Client, Error> {
    reqwest::blocking::Client::builder()
      // .proxy(reqwest::Proxy::https("http://localhost:8888")?) // Charles proxy
      .build()
  }
}

impl TcnApi for TcnApiImpl {

  fn get_reports(&self, interval_number: u64, interval_length: u64) -> Result<Vec<String>, NetworkingError> {
    println!("RUST downloading reports: interval: {}, length: {}", interval_number, interval_length);

    let url: &str = BASE_URL;
    let client = Self::create_client()?;
    let response = client.get(url)
      .header("Content-Type", "application/json")
      .query(&[("intervalNumber", interval_number)])
      .query(&[("intervalLength", interval_length)]) 
      .send()?;
    let reports = response.json::<Vec<String>>()?;
    println!("RUST retrieved reports count: {}", reports.len());
    Ok(reports)
  }

  fn post_report(&self, report: String) -> Result<(), NetworkingError> {
    println!("RUST posting report: {}", report);

    let url: &str = BASE_URL;
    let client = Self::create_client()?;
    let response = client.post(url)
      .header("Content-Type", "application/json")
      .body(report)
      .send()?;

    println!("RUST post report success: {:?}", response);
    Ok(response).map(|_| ())
  }
}

#[derive(Debug)]
pub struct NetworkingError {
  pub http_status: u16,
  pub message: String
}

impl fmt::Display for NetworkingError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "{:?}", self)
  }
}

impl From<Error> for NetworkingError {
  fn from(error: Error) -> Self {
    NetworkingError { 
      http_status: error
        .status()
        .map(|s| s.as_u16())
        .unwrap_or(UNKNOWN_HTTP_STATUS), 
      message: error.to_string()
    }
  }
}

impl error::Error for NetworkingError { }

// Convenience to map non-success HTTP status to errors
trait AsResult {
  fn as_result(self) -> Result<Response, NetworkingError>;
}

impl AsResult for Response {
  fn as_result(self) -> Result<Response, NetworkingError> {
    let status = self.status();
    if status.is_success() {
      Ok(self)
    } else {
      Err(NetworkingError { http_status: status.as_u16(), message: format!("{:?}", self.text()) })
    } 
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_reports_is_ok() {
    let api = TcnApiImpl {};
    let res = api.get_reports(73673, 21600);
    assert!(res.is_ok());
  }

  #[test]
  fn post_report_is_ok() {
    let api = TcnApiImpl {};
    let res = api.post_report("rSqWpM3ZQm7hfQ3q2x2llnFHiNhyRrUQPKEtJ33VKQcwT7Ly6e4KGaj5ZzjWt0m4c0v5n/VH5HO9UXbPXvsQTgEAQQAALFVtMVdNbHBZU1hOSlJYaDJZek5OWjJJeVdXZFpXRUozV2xoU2NHUkhWVDA9jn0pZAeME6ZBRHJOlfIikyfS0Pjg6l0txhhz6hz4exTxv8ryA3/Z26OebSRwzRfRgLdWBfohaOwOcSaynKqVCg==".to_owned());
    assert!(res.is_ok());
  }
}
