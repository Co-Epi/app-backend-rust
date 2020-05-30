use crate::preferences::{Preferences, PreferencesMock, TckArray, TCK_SIZE_IN_BYTES };
use tcn::{TemporaryContactKey, ReportAuthorizationKey, MemoType, SignedReport, Error};
use std::{cmp, io::Cursor, sync::Arc};
use cmp::max;




pub trait TcnKeys {
  fn create_report(&self, report: Vec<u8>) -> Result<SignedReport, Error>;
}

pub struct TcnKeysImpl<PreferencesType: Preferences> {
  pub preferences: Arc<PreferencesType>,
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

  fn byte_vec_to_66_byte_array(bytes: Vec<u8>) -> TckArray {
    let mut array = [0; TCK_SIZE_IN_BYTES];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes); 
    TckArray{tck_byte_array: array}
  }

  fn bytes_to_rak(bytes: [u8; 32]) -> ReportAuthorizationKey {
    ReportAuthorizationKey::read(Cursor::new(&bytes))
      .expect("Couldn't read RAK bytes")
  }

  fn tck_to_bytes(tck: TemporaryContactKey) -> TckArray {
    let mut buf = Vec::new();
    tck.write(Cursor::new(&mut buf))
      .expect("Couldn't write TCK bytes");
    Self::byte_vec_to_66_byte_array(buf)
  }

  fn bytes_to_tck(tck: TckArray) -> TemporaryContactKey {
    TemporaryContactKey::read(Cursor::new(&tck))
      .expect("Couldn't read TCK bytes")
  }
}



#[test]
fn test_rak(){
  let new_key = ReportAuthorizationKey::new(rand::thread_rng());
  // println!("{}", new_key);

  let tcn_key_impl = TcnKeysImpl {preferences: PreferencesMock {}};
  let bytes =  TcnKeysImpl::<PreferencesMock>::rak_to_bytes(new_key);
  println!("{:?}", bytes);
  assert_eq!(1,1);
}

#[test]
fn test_load_rak(){
  let bytes = [42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195, 126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85];
  let key = TcnKeysImpl::<PreferencesMock>::bytes_to_rak(bytes);

  let tck = key.initial_temporary_contact_key();

  println!("tck initial: {:#?}", tck);

  let tck_bytes = TcnKeysImpl::<PreferencesMock>::tck_to_bytes(tck);

  println!("Bytes: {:#?}", tck_bytes);

}

#[test]
fn test_load_tck(){
//use sha2::{Digest, Sha256};
  let rak_bytes = [42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195, 126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85];
  let key = TcnKeysImpl::<PreferencesMock>::bytes_to_rak(rak_bytes);
  // let rvk = tcn::ed25519_zebra::PublicKeyBytes::from(&key);
  

  let bytes = [34, 166, 47, 23, 224, 52, 240, 95, 140, 186, 95, 243, 26, 13, 174, 128, 224, 229, 158, 248, 117, 7, 118, 110, 108, 57, 67, 206, 129, 22, 84, 13];
  println!("count = {}", bytes.len());

  let version_bytes: [u8;2] = [1,0];

  let version_vec = version_bytes.to_vec();
  let rak_vec = rak_bytes.to_vec();
  let last_vec = bytes.to_vec();

  let mut complete_vec = [&version_vec[..], &rak_vec[..], &last_vec[..]].concat();

  // let bytes_array_complete: [u8; TCK_SIZE_IN_BYTES] = TcnKeysImpl::<PreferencesMock>::byte_vec_to_66_byte_array(complete_vec);

  let tck_array_struct = TcnKeysImpl::<PreferencesMock>::byte_vec_to_66_byte_array(complete_vec);

  let tck = TcnKeysImpl::<PreferencesMock>::bytes_to_tck(tck_array_struct);

  println!("{:#?}", tck);

}
