use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use crate::reports_interval::ReportsInterval;
use std::fmt;

pub const TCK_SIZE_IN_BYTES: usize = 66; 

pub enum PreferencesKey {
  LastCompletedReportsInterval
}

big_array! { BigArray; TCK_SIZE_IN_BYTES}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MyConfig {
  last_completed_reports_interval: Option<ReportsInterval>,
  autorization_key: Option<[u8; 32]>,
  tck: Option<TckBytesWrapper>,
}
//Wrapper struct added to enable custom serialization of a large byte array
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

pub struct PreferencesMock{
  pub rak_bytes: [u8; 32],
  pub tck_bytes: TckBytesWrapper
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
    let bytes = [42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195, 126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85];
    return Option::Some(bytes)
  }

  fn set_autorization_key(&self, _: [u8; 32]) { 
    return; 
  }

  fn tck(&self) -> std::option::Option<TckBytesWrapper> { 
    Some(self.tck_bytes)
  }

  fn set_tck(&self, value: TckBytesWrapper) { 
    return;
  }

}

