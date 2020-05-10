// Just some quick & dirty tests to check that things are working
// NOTE: Tests have to be run individually. Deleting & creating the db file doesn't work with cargo test
// We get Err(Io(Os { code: 17, kind: AlreadyExists, message: "File exists" }))
// TODO better tests. Don't use file.

// use tcn_client::*;
use std::{path::Path};
use tcn::TemporaryContactNumber;
use super::*;
use std::fs;

fn inits_db() {
let _ = fs::remove_file("./my-file");
let res = init(&Path::new("./my-file"));
}
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
      assert_eq!(stored_tcn, u128_of_tcn(tcn));
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
      assert_eq!(stored_tcn1, u128_of_tcn(tcn1));
      let stored_tcn2 = stored_tcns[1];
      assert_eq!(stored_tcn2, u128_of_tcn(tcn2));
    },
    Err(error) => println!("Stored TCNs error: {:?}", error)
  }; 
}

