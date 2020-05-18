// Just some quick & dirty tests to check that things are working
// NOTE: Tests have to be run individually. Deleting & creating the db file doesn't work with cargo test
// We get Err(Io(Os { code: 17, kind: AlreadyExists, message: "File exists" }))
// TODO better tests. Don't use file.

// use tcn_client::*;
use std::path::Path;
use tcn::TemporaryContactNumber;
use super::*;
use std::fs;
use tcn::SignedReport;
use base64::DecodeError;

#[test]
fn inits_db() {
  let _ = fs::remove_file("./tcn-db");

  let res = init(&Path::new("./tcn-db"));
  println!("Init DB res: {:?}", res);

  match &res {
    Ok(success) => println!("Init DB success: {:?}", success),
    Err(error) => println!("Init DB error: {:?}", error)
  };

  assert!(res.is_ok());
  // assert!(matches!(res, Ok(_)));
}

#[test]
fn stores_tcn() {
  let _ = fs::remove_file("./tcn-db");

  let res = init(&Path::new("./tcn-db"));
  println!("Init DB res: {:?}", res);
  assert!(res.is_ok());

  let tcn = TemporaryContactNumber([1;16]);
  let store_res = record_tcn(tcn);

  println!("store_res: {:?}", store_res);

  assert!(store_res.is_ok());

  let stored_tcns_res = all_stored_tcns();

  assert!(stored_tcns_res.is_ok());

  match &stored_tcns_res {
    Ok(stored_tcns) => {
      println!("Stored TCNs: {:?}", stored_tcns);
      assert_eq!(stored_tcns.len(), 1);

      let stored_tcn = stored_tcns[0];
      assert_eq!(stored_tcn, u128_of_tcn(&tcn));
    },
    Err(error) => println!("Stored TCNs error: {:?}", error)
  }; 
}


#[test]
fn stores_multiple_tcns() {
  let _ = fs::remove_file("./tcn-db");

  let res = init(&Path::new("./tcn-db"));
  println!("Init DB res: {:?}", res);
  assert!(res.is_ok());

  let tcn1 = TemporaryContactNumber([1;16]);
  let tcn2 = TemporaryContactNumber([2;16]);
  let store_res1 = record_tcn(tcn1);
  let store_res2 = record_tcn(tcn2);

  println!("store_res: {:?}", store_res1);

  assert!(store_res1.is_ok());
  assert!(store_res2.is_ok());

  let stored_tcns_res = all_stored_tcns();

  assert!(stored_tcns_res.is_ok());

  match &stored_tcns_res {
    Ok(stored_tcns) => {
      println!("Stored TCNs: {:?}", stored_tcns);
      assert_eq!(stored_tcns.len(), 2);

      let stored_tcn1 = stored_tcns[0];
      assert_eq!(stored_tcn1, u128_of_tcn(&tcn1));
      let stored_tcn2 = stored_tcns[1];
      assert_eq!(stored_tcn2, u128_of_tcn(&tcn2));
    },
    Err(error) => println!("Stored TCNs error: {:?}", error)
  }; 
}

#[test]
fn matches_tcn() {

  // Generate a TCN from report
  let report_str = "rSqWpM3ZQm7hfQ3q2x2llnFHiNhyRrUQPKEtJ33VKQcwT7Ly6e4KGaj5ZzjWt0m4c0v5n/VH5HO9UXbPXvsQTgEAQQAALFVtMVdNbHBZU1hOSlJYaDJZek5OWjJJeVdXZFpXRUozV2xoU2NHUkhWVDA9jn0pZAeME6ZBRHJOlfIikyfS0Pjg6l0txhhz6hz4exTxv8ryA3/Z26OebSRwzRfRgLdWBfohaOwOcSaynKqVCg==";
  let decoded: Result<Vec<u8>, DecodeError> = base64::decode(report_str);
  let signed_report = SignedReport::read( decoded.unwrap().as_slice()).unwrap();
  let report = signed_report.verify().unwrap();
  let tcn: TemporaryContactNumber = report.temporary_contact_numbers().next().unwrap();

  // Check that report matches with generated tcn
  let tcns: HashSet<u128> = [tcn].into_iter().map(|t| u128_of_tcn(t)).collect();
  let res = match_reports_with(tcns, vec![&report].into_iter());

  assert!(res.is_ok());
 }
