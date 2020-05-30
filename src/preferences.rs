use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use crate::reports_interval::ReportsInterval;
use crate::tcn_ext::tcn_keys::TcnKeysImpl;
use std::fmt;

pub const TCK_SIZE_IN_BYTES: usize = 66; 

const USER_LIMIT:i32 = 100; 
pub enum PreferencesKey {
  LastCompletedReportsInterval
}

big_array! { BigArray; +TCK_SIZE_IN_BYTES}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MyConfig {
  last_completed_reports_interval: Option<ReportsInterval>,
  autorization_key: Option<[u8; 32]>,
  tck: Option<TckArray>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TckArray {
  #[serde(with = "BigArray")]
  pub tck_byte_array: [u8; TCK_SIZE_IN_BYTES]
}

impl fmt::Debug for TckArray {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      self.tck_byte_array[..].fmt(formatter)
  }
}

/*
error[E0277]: the trait bound `preferences::TckArray: std::convert::AsRef<[u8]>` is not satisfied
   --> src/tcn_ext/tcn_keys.rs:87:31
    |
87  |     TemporaryContactKey::read(Cursor::new(&tck))
    |                               ^^^^^^^^^^^^^^^^^ the trait `std::convert::AsRef<[u8]>` is not implemented for `preferences::TckArray`
    | 
   ::: /Users/duskoo/.cargo/git/checkouts/tcn-c61a3c0b0375ae97/37add39/src/serialize.rs:128:20
    |
128 |     pub fn read<R: io::Read>(mut reader: R) -> Result<TemporaryContactKey, io::Error> {
    |                    -------- required by this bound in `tcn::serialize::<impl tcn::keys::TemporaryContactKey>::read`
    |
    = note: required because of the requirements on the impl of `std::convert::AsRef<[u8]>` for `&preferences::TckArray`
    = note: required because of the requirements on the impl of `std::io::Read` for `std::io::Cursor<&preferences::TckArray>`

    */


impl AsRef<[u8]> for TckArray {
  fn as_ref(&self) -> &[u8] {
      &self.tck_byte_array//.first().unwrap()//?? https://stackoverflow.com/q/29278940
  }
}

// fn from_big_array<D>(deserializer: D) -> Result<[u8;TCK_SIZE_IN_BYTES], D::Error>
// where d:Deserializer
// {

// }



// impl Deserialize for MyConfig {
//   fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//   where D: Deserializer<'de> {
//       let mut myConfig = MyConfig{
//         last_completed_reports_interval: None,
//         autorization_key: None,
//         tck: None
//       };


//   }
// }

impl Default for MyConfig {
  fn default() -> Self { Self { 
    last_completed_reports_interval: None,
    autorization_key: None,
    tck: None,
  }}
}

// TODO either change storage (confy) and use api more similar to Android/iOS (using generic functions with keys)
// TODO or remove PreferencesKey
pub trait Preferences {
  fn last_completed_reports_interval(&self, key: PreferencesKey) -> Option<ReportsInterval>;
  fn set_last_completed_reports_interval(&self, key: PreferencesKey, value: ReportsInterval);


  // TODO encrypted
  fn authorization_key(&self) -> Option<[u8; 32]>;
  fn set_autorization_key(&self, value: [u8; 32]);

  fn tck(&self) -> Option<TckArray>;
  fn set_tck(&self, value: TckArray);
}

pub struct PreferencesImpl {
  pub config: RwLock<MyConfig>
}

impl Preferences for PreferencesImpl {

  fn last_completed_reports_interval(&self, key: PreferencesKey) -> Option<ReportsInterval> {
    match key {
      LastCompletedReportsInterval => self.config.read().last_completed_reports_interval
    }
  }

  fn set_last_completed_reports_interval(&self, key: PreferencesKey, value: ReportsInterval) {
    let mut config = self.config.write();
    config.last_completed_reports_interval = Some(value);

    let res = confy::store("myprefs", *config);
  
    if let Err(error) = res {
      println!("Error storing preferences: {:?}", error)
    }
  }

  fn authorization_key(&self) -> Option<[u8; 32]> {
    self.config.read().autorization_key
  }

  fn set_autorization_key(&self, value: [u8; 32]) {
    let mut config = self.config.write();
    config.autorization_key = Some(value);

    let res = confy::store("myprefs", *config);
  
    if let Err(error) = res {
      println!("Error storing preferences: {:?}", error)
    }
  }

  fn tck(&self) -> Option<TckArray> {
    self.config.read().tck
  }

  fn set_tck(&self, value: TckArray) {
    let mut config = self.config.write();
    config.tck = Some(value);

    let res = confy::store("myprefs", *config);
  
    if let Err(error) = res {
      println!("Error storing preferences: {:?}", error)
    }
  }
}

pub struct PreferencesMock;

impl Preferences for PreferencesMock {
  fn last_completed_reports_interval(&self, _: PreferencesKey) -> std::option::Option<ReportsInterval> { 
    let reports_interval = ReportsInterval{number: 8899222, length: 12232 };
    return Option::Some(reports_interval)

  }

  fn set_last_completed_reports_interval(&self, _: PreferencesKey, _: ReportsInterval) { 
    return;
  }

  fn authorization_key(&self) -> std::option::Option<[u8; 32]> { 
    /*
    let new_key = ReportAuthorizationKey::new(rand::thread_rng());
      self.preferences.set_autorization_key(Self::rak_to_bytes(new_key));
    */
    let bytes = [42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195, 126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85];
    // let authorization_key = TcnKeysImpl::<PreferencesMock>::bytes_to_rak(bytes);
    return Option::Some(bytes)
  }

  fn set_autorization_key(&self, _: [u8; 32]) { 
    return; 
  }

  fn tck(&self) -> std::option::Option<TckArray> { 
    todo!() 
  }

  fn set_tck(&self, value: TckArray) { 
    todo!() 
  }
}

