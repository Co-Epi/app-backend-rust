use crate::reports_interval::UnixTime;
use std::{u64, convert::TryInto};

struct Memo {
  bytes: Vec<u8>
}
#[derive(Debug, PartialEq, Clone)]
enum UserInput<T> {
  Some(T),
  None
}

#[derive(Debug, PartialEq, Clone)]
enum FeverSeverity {
  None, Mild, Serious
}
#[derive(Debug, PartialEq, Clone)]
enum CoughSeverity {
  None, Existing, Wet, Dry
}

#[derive(Debug, PartialEq, Clone)]
struct PublicReport {
  earliest_symptom_time: UserInput<UnixTime>,
  fever_severity: FeverSeverity,
  cough_severity: CoughSeverity,
  breathlessness: bool
}

impl PublicReport {
  fn should_be_sent(&self) -> bool {
    self.fever_severity != FeverSeverity::None || self.cough_severity != CoughSeverity::None || self.breathlessness
  }
}

trait MemoMapper {
  fn to_memo(report: PublicReport, time: UnixTime) -> Memo;
  fn to_report(memo: Memo) -> PublicReport;
}

struct MemoMapperImpl {
}

impl MemoMapperImpl {
  const VERSION_MAPPER: VersionMapper = VersionMapper {};
  const TIME_MAPPER: TimeMapper = TimeMapper {};
  const TIME_USER_INPUT_MAPPER: TimeUserInputMapper = TimeUserInputMapper {};
  const COUGH_SEVERITY_MAPPER: CoughSeverityMapper = CoughSeverityMapper {};
  const FEVER_SEVERITY_MAPPER: FeverSeverityMapper = FeverSeverityMapper {};
  const BOOLEAN_MAPPER: BoolMapper = BoolMapper {};

  fn extract<T>(bits: &Vec<bool>, mapper: &dyn BitMapper<T>, start: usize) -> ExtractResult<T> {
    let end = mapper.bit_count() + start;
    let sub_bits: Vec<bool> = bits[start..end].try_into().expect("msg");

    ExtractResult { 
      value: mapper.from_bits(BitList { bits: sub_bits }), 
      count: mapper.bit_count() 
    }
  }
}

impl MemoMapper for MemoMapperImpl {

    fn to_memo(report: PublicReport, time: UnixTime) -> Memo {

      let memo_version: u16 = 1;

      let bits = vec![
        Self::VERSION_MAPPER.to_bits(memo_version),
        Self::TIME_MAPPER.to_bits(time),
        Self::TIME_USER_INPUT_MAPPER.to_bits(report.earliest_symptom_time),
        Self::COUGH_SEVERITY_MAPPER.to_bits(report.cough_severity),
        Self::FEVER_SEVERITY_MAPPER.to_bits(report.fever_severity),
        Self::BOOLEAN_MAPPER.to_bits(report.breathlessness),
      ];

      Memo { bytes: bits.into_iter().fold(BitList { bits: vec![] }, |acc, e|
        acc.concat(e)
      ).as_u8_array() }
    }

    fn to_report(memo: Memo) -> PublicReport {
      let bits: Vec<bool> = memo.bytes.into_iter().flat_map(|byte|
        byte.to_bits().bits
      ).collect();

      let mut next: usize = 0;


      // Version for now not handled
      let version_result = Self::extract(&bits, &Self::VERSION_MAPPER, next)
        .value (|v| next += v);

      // TODO handle report time?
      let time_result = Self::extract(&bits, &Self::TIME_MAPPER, next)
        .value (|v| next += v);

      let earliest_symptom_time = Self::extract(&bits, &Self::TIME_USER_INPUT_MAPPER, next)
        .value (|v| next += v);
      let cough_severity = Self::extract(&bits, &Self::COUGH_SEVERITY_MAPPER, next)
        .value (|v| next += v);
      let fever_severity = Self::extract(&bits, &Self::FEVER_SEVERITY_MAPPER, next)
        .value (|v| next += v);
      let breathlessness = Self::extract(&bits, &Self::BOOLEAN_MAPPER, next)
        .value (|v| next += v);

      PublicReport {
        earliest_symptom_time,
        fever_severity,
        cough_severity,
        breathlessness
      }
    }
}

struct ExtractResult<T>{ value: T, count: usize }

impl <T> ExtractResult<T> {
  // Convenience to parse memo with less boilerplate
  fn value<F: FnOnce(usize) -> ()>(self, f: F) -> T {
    f(self.count);
    self.value
  }
}

trait BitMapper<T> {

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


struct VersionMapper {}
impl BitMapper<u16> for VersionMapper {

  fn bit_count(&self) -> usize { 16 }

  fn to_bits_unchecked(&self, value: u16) -> BitList {
    value.to_bits()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> u16 {
    bit_list.as_u16()
  }
}

struct TimeMapper {}
impl BitMapper<UnixTime> for TimeMapper {

  fn bit_count(&self) -> usize { 64 }

  fn to_bits_unchecked(&self, value: UnixTime) -> BitList {
    value.value.to_bits()
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> UnixTime {
    UnixTime { value: bit_list.as_u64() }
  }
}

struct TimeUserInputMapper {}
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

struct BoolMapper {}
impl BitMapper<bool> for BoolMapper {

  fn bit_count(&self) -> usize { 1 }

  fn to_bits_unchecked(&self, value: bool) -> BitList {
    BitList { bits: vec![value] }
  }

  fn from_bits_unchecked(&self, bit_list: BitList) -> bool {
    bit_list.bits.first().unwrap().to_owned()
  }
}

struct CoughSeverityMapper {}
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

struct FeverSeverityMapper {}
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

trait BitListMappable {
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

struct BitList {
  bits: Vec<bool>
}

impl BitList {

  fn concat(&self, bitList: BitList) -> BitList {
    BitList { bits: self.bits.clone().into_iter().chain(bitList.bits.into_iter()).collect() }
  }

  fn as_u8_array(&self) -> Vec<u8> {
    let chunks: Vec<[bool; 8]> = Self::as_byte_chunks(self.bits.clone());
    chunks.into_iter().map(|chunk| {
      let bit_string = Self::to_bit_string(chunk);
      let reversed: String = bit_string.chars().rev().collect(); // Most significant bit first
      u8::from_str_radix(reversed.as_ref(), 2).unwrap()
    })
    .collect()
  }

  fn to_bit_string(bits: [bool; 8]) -> String {
    let strs: Vec<&str> = bits.clone().into_iter().map(|bit: &bool|
      if *bit { "1" } else { "0" }
    ).collect();

    strs.join("")
  }

  fn as_byte_chunks(bits: Vec<bool>) -> Vec<[bool; 8]> {
    Self::fill_until_size(
      bits.clone(), 
      ((bits.len() as f32) / 8.0).ceil() as usize * 8, 
      false
    )
    .chunks(8)
    .map(|x | x.try_into().expect("Chunk doesn't have 8 bits"))
    .collect()
  }

  fn fill_until_size<T: Copy>(vec: Vec<T>, size: usize, with: T) -> Vec<T> {
    let mut mut_vec = vec;
    while mut_vec.len() < size {
      mut_vec.push(with);
    }
    mut_vec
  }

  fn len(&self) -> usize {
    self.bits.len()
  }

  fn as_u8(&self) -> u8 {
    let arr: [u8; 1] = self.as_u8_array().as_slice().try_into()
    .expect("Unexpected size");
    unsafe { std::mem::transmute::<[u8; 1], u8>(arr) }
  }

  fn as_u16(&self) -> u16 {
    let arr: [u8; 2] = self.as_u8_array().as_slice().try_into()
    .expect("Unexpected size");
    unsafe { std::mem::transmute::<[u8; 2], u16>(arr) }
  }

  fn as_u64(&self) -> u64 {
    let arr: [u8; 8] = self.as_u8_array().as_slice().try_into()
    .expect("Unexpected size");
    unsafe { std::mem::transmute::<[u8; 8], u64>(arr) }
  }

  fn as_unibble_bit_list(&self) -> BitList {
    BitList { bits: Self::fill_until_size(self.bits.clone(), 4, false).into_iter().take(4).collect() }
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

  #[test]
  fn maps_no_symptoms() {
    let report = PublicReport {
        earliest_symptom_time: UserInput::None,
        fever_severity: FeverSeverity::None,
        breathlessness: false,
        cough_severity: CoughSeverity::None
    };

    let memo: Memo = MemoMapperImpl::to_memo(report.clone(), UnixTime { value: 1589209754 });
    let mapped_report: PublicReport = MemoMapperImpl::to_report(memo);

    assert_eq!(mapped_report, report.clone());
  }

  #[test]
  fn maps_all_symptoms_set_arbitrary() {
    let report = PublicReport {
        earliest_symptom_time: UserInput::Some(UnixTime { value: 1589209754 }),
        fever_severity: FeverSeverity::Serious,
        breathlessness: true,
        cough_severity: CoughSeverity::Existing
    };

    let memo: Memo = MemoMapperImpl::to_memo(report.clone(), UnixTime { value: 0 });
    let mapped_report: PublicReport = MemoMapperImpl::to_report(memo);

    assert_eq!(mapped_report, report.clone());
  }
}
