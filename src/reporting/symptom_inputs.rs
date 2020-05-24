use std::{io::Cursor, collections::HashSet};
use crate::{errors::ServicesError, reports_interval::UnixTime, tcn_ext::tcn_keys::TcnKeys, networking::TcnApi};
use serde::Deserialize;
use super::{memo::MemoMapper, public_report::PublicReport};
use tcn::SignedReport;

#[derive(Debug, Deserialize)]
pub struct SymptomInputs {
  pub ids: HashSet<SymptomId>,
  pub cough: Cough,
  pub breathlessness: Breathlessness,
  pub fever: Fever,
  pub earliest_symptom: EarliestSymptom
}

#[derive(Debug, Deserialize)]
pub struct Cough {
  pub cough_type: UserInput<CoughType>,
  pub days: UserInput<Days>,
  pub status: UserInput<CoughStatus>
}

#[derive(Debug, Deserialize)]
pub enum CoughType {
  Wet, Dry
}

#[derive(Debug, Deserialize)]
pub enum CoughStatus {
  BetterAndWorseThroughDay, WorseWhenOutside, SameOrSteadilyWorse
}

#[derive(Debug, Deserialize)]
pub struct Days {
  pub value: i32
}

#[derive(Debug, Deserialize)]
pub struct Breathlessness {
  pub cause: UserInput<BreathlessnessCause>
}

#[derive(Debug, Deserialize)]
pub enum BreathlessnessCause {
  LeavingHouseOrDressing, WalkingYardsOrMinsOnGround, GroundOwnPace, HurryOrHill, Exercise
}

#[derive(Debug, Deserialize)]
pub struct Fever {
  pub days: UserInput<Days>,
  pub taken_temperature_today: UserInput<bool>,
  pub temperature_spot: UserInput<TemperatureSpot>,
  pub highest_temperature: UserInput<FarenheitTemperature>
}

#[derive(Debug, Deserialize)]
pub enum TemperatureSpot {
  Mouth, Ear, Armpit, Other(String)
}

// Temperature conversions are only for presentation, so in the apps
#[derive(Debug, Deserialize)]
pub struct FarenheitTemperature {
  pub value: f32
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum SymptomId {
  Cough, Breathlessness, Fever, MuscleAches, LossSmellOrTaste, Diarrhea, RunnyNose, Other, None
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum UserInput<T> {
  Some(T),
  None
}

#[derive(Debug, Deserialize)]
pub struct EarliestSymptom {
  pub time: UserInput<UnixTime>
}

pub trait SymptomInputsSubmitter<
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> {
  fn submit_inputs(&self, inputs: SymptomInputs) -> Result<(), ServicesError>;
}

pub struct SymptomInputsSubmitterImpl<'a,
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> {
  pub memo_mapper: MemoMapperType,
  pub tcn_keys: TcnKeysType,
  pub api: &'a TcnApiType,
}

impl <'a,
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> SymptomInputsSubmitter<
  MemoMapperType, TcnKeysType, TcnApiType
> for SymptomInputsSubmitterImpl<'a,
  MemoMapperType, TcnKeysType, TcnApiType
> {

  fn submit_inputs(&self, inputs: SymptomInputs) -> Result<(), ServicesError> {
    let public_report = PublicReport::with_inputs(inputs);

    if !public_report.should_be_sent() {
      println!("Public report: {:?} doesn't contain infos relevant to other users. Not sending.", public_report);
      return Ok(())
    }

    let memo = self.memo_mapper.to_memo(public_report, UnixTime::now());

    let signed_report = self.tcn_keys.crate_report(memo.bytes)?;

    let report_str = base64::encode(signed_report_to_bytes(signed_report));
    
    self.api.post_report(report_str).map_err(ServicesError::from)
  }
}

fn signed_report_to_bytes(signed_report: SignedReport) -> Vec<u8> {
  let mut buf = Vec::new();
  signed_report.write(Cursor::new(&mut buf)).expect("Couldn't write signed report bytes");
  buf
}
