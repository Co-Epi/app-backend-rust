use serde::{Serialize, Deserialize};
use parking_lot::RwLock;
use crate::reports_interval::ReportsInterval;

pub enum PreferencesKey {
  LastCompletedReportsInterval
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MyConfig {
  last_completed_reports_interval: Option<ReportsInterval>,
  autorization_key: Option<[u8; 32]>,
  tck: Option<[u8; 32]>,
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

  fn tck(&self) -> Option<[u8; 32]>;
  fn set_tck(&self, value: [u8; 32]);
}

pub struct PreferencesImpl {
  pub config: RwLock<MyConfig>
}

impl Preferences for PreferencesImpl {

  fn last_completed_reports_interval(&self, key: PreferencesKey) -> Option<ReportsInterval> {
    match key {
      PreferencesKey::LastCompletedReportsInterval => self.config.read().last_completed_reports_interval
    }
  }

  fn set_last_completed_reports_interval(&self, _: PreferencesKey, value: ReportsInterval) {
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

  fn tck(&self) -> Option<[u8; 32]> {
    self.config.read().tck
  }

  fn set_tck(&self, value: [u8; 32]) {
    let mut config = self.config.write();
    config.tck = Some(value);

    let res = confy::store("myprefs", *config);
  
    if let Err(error) = res {
      println!("Error storing preferences: {:?}", error)
    }
  }
}
