use crate::preferences::Preferences;
use tcn::{TemporaryContactKey, ReportAuthorizationKey, MemoType, SignedReport, Error};
use std::{cmp, io::Cursor};
use cmp::max;

pub trait TcnKeys {
  fn create_report(&self, report: Vec<u8>) -> Result<SignedReport, Error>;
}

pub struct TcnKeysImpl<PreferencesType: Preferences> {
  pub preferences: PreferencesType,
}

impl <PreferencesType: Preferences> TcnKeys for TcnKeysImpl<PreferencesType> {
  fn create_report(&self, report: Vec<u8>) -> Result<SignedReport, Error> {
    let end_index = self.tck().index();
    let minutes_in_14_days: u32 = 14 * 24 * 60 * 60;
    let periods = minutes_in_14_days / 15;
    let start_index = max(0, (end_index as u32) - periods) as u16;

    self.rak().create_report(MemoType::CoEpiV1, report, start_index, end_index)
  }
}

impl <PreferencesType: Preferences> TcnKeysImpl<PreferencesType> {
  fn rak(&self) -> ReportAuthorizationKey {
    self.preferences.authorization_key().map(|rak_bytes|
      Self::bytes_to_rak(rak_bytes)

    ).unwrap_or({
      let new_key = ReportAuthorizationKey::new(rand::thread_rng());
      self.preferences.set_autorization_key(Self::rak_to_bytes(new_key));
      new_key
    })
  }

  fn tck(&self) -> TemporaryContactKey {
    self.preferences.tck().map(|rak_bytes|
      Self::bytes_to_tck(rak_bytes)

    ).unwrap_or({
      self.rak().initial_temporary_contact_key()
    })
  }

  fn set_tck(&self, tck: TemporaryContactKey) {
    self.preferences.set_tck(Self::tck_to_bytes(tck));
  }

  fn rak_to_bytes(rak: ReportAuthorizationKey) -> [u8; 32] {
    let mut buf = Vec::new();
    rak.write(Cursor::new(&mut buf))
      .expect("Couldn't write RAK bytes");
    Self::byte_vec_to_32_byte_array(buf)
  }

  fn byte_vec_to_32_byte_array(bytes: Vec<u8>) -> [u8; 32] {
    let mut array = [0; 32];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes); 
    array
  }

  fn bytes_to_rak(bytes: [u8; 32]) -> ReportAuthorizationKey {
    ReportAuthorizationKey::read(Cursor::new(&bytes))
      .expect("Couldn't read RAK bytes")
  }

  fn tck_to_bytes(tck: TemporaryContactKey) -> [u8; 32] {
    let mut buf = Vec::new();
    tck.write(Cursor::new(&mut buf))
      .expect("Couldn't write TCK bytes");
    Self::byte_vec_to_32_byte_array(buf)
  }

  fn bytes_to_tck(bytes: [u8; 32]) -> TemporaryContactKey {
    TemporaryContactKey::read(Cursor::new(&bytes))
      .expect("Couldn't read TCK bytes")
  }
}
