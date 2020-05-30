use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use crate::reports_interval::ReportsInterval;
use crate::tcn_ext::tcn_keys::TcnKeysImpl;
use std::fmt;
use tcn::TemporaryContactKey;

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
  tck: Option<TckBytesWrapper>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TckBytesWrapper {
  #[serde(with = "BigArray")]
  pub tck_bytes: [u8; TCK_SIZE_IN_BYTES]
}

impl fmt::Debug for TckBytesWrapper {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      self.tck_bytes[..].fmt(formatter)
  }
}




impl AsRef<[u8]> for TckBytesWrapper {
  fn as_ref(&self) -> &[u8] {
      &self.tck_bytes
  }
}



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

  fn tck(&self) -> Option<TckBytesWrapper>;
  fn set_tck(&self, value: TckBytesWrapper);
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

  fn tck(&self) -> Option<TckBytesWrapper> {
    self.config.read().tck
  }

  fn set_tck(&self, value: TckBytesWrapper) {
    let mut config = self.config.write();
    config.tck = Some(value);

    let res = confy::store("myprefs", *config);
  
    if let Err(error) = res {
      println!("Error storing preferences: {:?}", error)
    }
  }
}

pub struct PreferencesMock;

impl PreferencesMock {
  fn generate_tck(&self, index: usize) -> Option<TemporaryContactKey>{
    if let Some(rak_bytes) = self.authorization_key() {
      let rak = TcnKeysImpl::<PreferencesMock>::bytes_to_rak(rak_bytes);
      let mut tck = rak.initial_temporary_contact_key(); // tck <- tck_1
      // let mut tcns = Vec::new();
      for _ in 0..index {
        // tcns.push(tck.temporary_contact_number());
        tck = tck.ratchet().unwrap();
      }
  
      return Some(tck)
  
    }else{
      return None
    }
  
  }
}

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

  fn tck(&self) -> std::option::Option<TckBytesWrapper> { 
    if let Some(tck) = self.generate_tck(15){
      let tck_bytes = TcnKeysImpl::<PreferencesMock>::tck_to_bytes(tck);
      return Some(tck_bytes)
    }else{
      return None
    }
  }

  fn set_tck(&self, value: TckBytesWrapper) { 
    return;
  }

}

