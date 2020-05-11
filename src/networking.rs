use reqwest::{blocking::{Client, Response}, Error};

static BASE_URL: &str = "https://18ye1iivg6.execute-api.us-west-1.amazonaws.com/v4";

fn get_reports(interval_number: u32, interval_length: u32) -> Result<Vec<String>, Error> {
  let url: &str = &format!("{}/tcnreport", BASE_URL);

  create_client().and_then(|client| 
      client.get(url)
          .header("Content-Type", "application/json")
          .query(&[("intervalNumber", interval_number)])
          // TODO: Will be changed to seconds
          .query(&[("intervalLengthMs", interval_length)])
          .send()
          .and_then (|response| response.json::<Vec<String>>())
  )
}

fn post_report(report: &'static str) -> Result<Response, Error> {
  let url: &str = &format!("{}/tcnreport", BASE_URL);

  create_client().and_then(|client| 
      client.post(url)
          .header("Content-Type", "application/json")
          .body(report)
          .send()
  )
}

fn create_client() -> Result<Client, Error> {
  reqwest::blocking::Client::builder()
    .proxy(reqwest::Proxy::https("http://localhost:8888")?) // Charles
    .build()
}

#[cfg(test)]
mod tests {
  use super::*;
  use reqwest::StatusCode;

  #[test]
  fn get_reports_is_ok() {
    let res = get_reports(1, 21600000);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(),  Vec::<String>::new());
  }

  #[test]
  fn post_report_is_ok() {
    let res = post_report("rSqWpM3ZQm7hfQ3q2x2llnFHiNhyRrUQPKEtJ33VKQcwT7Ly6e4KGaj5ZzjWt0m4c0v5n/VH5HO9UXbPXvsQTgEAQQAALFVtMVdNbHBZU1hOSlJYaDJZek5OWjJJeVdXZFpXRUozV2xoU2NHUkhWVDA9jn0pZAeME6ZBRHJOlfIikyfS0Pjg6l0txhhz6hz4exTxv8ryA3/Z26OebSRwzRfRgLdWBfohaOwOcSaynKqVCg==");
    assert!(res.is_ok());
    assert_eq!(res.unwrap().status(), StatusCode::from_u16(200).unwrap());
  }
}
