use super::{public_report::{CoughSeverity, UserInput, FeverSeverity}, bit_list::BitList};
use crate::reports_interval::UnixTime;

pub trait BitMapper<T> {

  fn bit_count(&self) -> usize;

  fn to_bits(&self, value: T) -> BitList {
    let bits = self.to_bits_unchecked(value);
    if bits.len() != self.bit_count() {
      panic!("Incorrect bit count: {}. Required: {}", bits.len(), self.bit_count())
    } else {
      bits
    }
  }

  fn from_bits(&self, bit_list: BitList) -> T {
    if bit_list.len() != self.bit_count() {
      panic!("Incorrect bit count: {}. Required: {}", bit_list.len(), self.bit_count())
    }
    self.from_bits_unchecked(bit_list)
  }

  fn to_bits_unchecked(&self, value: T) -> BitList;

  fn from_bits_unchecked(&self, bit_list: BitList) -> T;
}

pub struct VersionMapper {}
impl BitMapper<u16> for VersionMapper {

  fn bit_count(&self) -> usize { 16 }

  fn to_bits_unchecked(&self, value: u16) -> BitList {
    value.to_bits()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> u16 {
    bit_list.as_u16()
  }
}

pub struct TimeMapper {}
impl BitMapper<UnixTime> for TimeMapper {

  fn bit_count(&self) -> usize { 64 }

  fn to_bits_unchecked(&self, value: UnixTime) -> BitList {
    value.value.to_bits()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> UnixTime {
    UnixTime { value: bit_list.as_u64() }
  }
}

pub struct TimeUserInputMapper {}
impl BitMapper<UserInput<UnixTime>> for TimeUserInputMapper {

  fn bit_count(&self) -> usize { 64 }

  fn to_bits_unchecked(&self, value: UserInput<UnixTime>) -> BitList {
    match value {
      UserInput::None => u64::MAX,
      UserInput::Some(input) => input.value
    }.to_bits()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> UserInput<UnixTime> {
    let value = bit_list.as_u64();
    match value {
      u64::MAX => UserInput::None,
      _ => UserInput::Some(UnixTime { value })
    }
  }
}

pub struct BoolMapper {}
impl BitMapper<bool> for BoolMapper {

  fn bit_count(&self) -> usize { 1 }

  fn to_bits_unchecked(&self, value: bool) -> BitList {
    BitList { bits: vec![value] }
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> bool {
    bit_list.bits.first().unwrap().to_owned()
  }
}

pub struct CoughSeverityMapper {}
impl BitMapper<CoughSeverity> for CoughSeverityMapper {

  fn bit_count(&self) -> usize { 4 }

  fn to_bits_unchecked(&self, value: CoughSeverity) -> BitList {
    (match value {
      CoughSeverity::None => 0,
      CoughSeverity::Existing => 1,
      CoughSeverity::Dry => 2,
      CoughSeverity::Wet => 3,
    } as u8).to_bits().as_unibble_bit_list()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> CoughSeverity {
    let value = bit_list.as_u8();
    match value {
      0 => CoughSeverity::None,
      1 => CoughSeverity::Existing,
      2 => CoughSeverity::Dry,
      3 => CoughSeverity::Wet,
      _ => panic!("Not supported: {}", value)
    }
  }
}

pub struct FeverSeverityMapper {}
impl BitMapper<FeverSeverity> for FeverSeverityMapper {

  fn bit_count(&self) -> usize { 4 }

  fn to_bits_unchecked(&self, value: FeverSeverity) -> BitList {
    (match value {
      FeverSeverity::None => 0,
      FeverSeverity::Mild => 1,
      FeverSeverity::Serious => 2,
    } as u8).to_bits().as_unibble_bit_list()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> FeverSeverity {
    let value = bit_list.as_u8();
    match value {
      0 => FeverSeverity::None,
      1 => FeverSeverity::Mild,
      2 => FeverSeverity::Serious,
      _ => panic!("Not supported: {}", value)
    }
  }
}

pub trait BitListMappable {
  fn to_bits(&self) -> BitList;
}

impl BitListMappable for u64 {
  fn to_bits(&self) -> BitList {
    let bits: Vec<bool> = (0..64).map(|index| {
      let value: Self = (self >> index) & 0x01;
      value == 1
    }).collect();
    BitList { bits }
  }
}

impl BitListMappable for u16 {
  fn to_bits(&self) -> BitList {
    let bits: Vec<bool> = (0..16).map(|index| {
      let value: Self = (self >> index) & 0x01;
      value == 1
    }).collect();
    BitList { bits }
  }
}

impl BitListMappable for u8 {
  fn to_bits(&self) -> BitList {
    let bits: Vec<bool> = (0..8).map(|index| {
      let value: Self = (self >> index) & 0x01;
      value == 1
    }).collect();
    BitList { bits }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn version_mapper_maps_10_to_bits() {
    let version_mapper = VersionMapper {};
    let bit_list = version_mapper.to_bits(10);

    let mut bits = vec![false; 16];
    bits[1] = true;
    bits[3] = true;

    assert_eq!(bits, bit_list.bits);
  }

  #[test]
  fn version_mapper_maps_bits_to_10() {
    let version_mapper = VersionMapper {};

    let mut bits = vec![false; 16];
    bits[1] = true;
    bits[3] = true;

    let number = version_mapper.from_bits(BitList { bits: bits} );

    assert_eq!(number, 10);
  }
}
