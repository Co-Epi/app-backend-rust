use std::{io::Cursor, collections::HashSet};
use crate::{errors::ServicesError, reports_interval::UnixTime, tcn_ext::tcn_keys::TcnKeys, networking::TcnApi};
use serde::Deserialize;
use super::{memo::MemoMapper, public_report::PublicReport};
use tcn::SignedReport;

#[derive(Deserialize)]
pub struct SymptomInputs {
  pub ids: HashSet<SymptomId>,
  pub cough: Cough,
  pub breathlessness: Breathlessness,
  pub fever: Fever,
  pub earliest_symptom: EarliestSymptom
}

#[derive(Deserialize)]
pub struct Cough {
  pub cough_type: UserInput<CoughType>,
  pub days: UserInput<Days>,
  pub status: UserInput<CoughStatus>
}

#[derive(Deserialize)]
pub enum CoughType {
  Wet, Dry
}

#[derive(Deserialize)]
pub enum CoughStatus {
  BetterAndWorseThroughDay, WorseWhenOutside, SameOrSteadilyWorse
}

#[derive(Deserialize)]
pub struct Days {
  pub value: i32
}

#[derive(Deserialize)]
pub struct Breathlessness {
  pub cause: UserInput<BreathlessnessCause>
}

#[derive(Deserialize)]
pub enum BreathlessnessCause {
  LeavingHouseOrDressing, WalkingYardsOrMinsOnGround, GroundOwnPace, HurryOrHill, Exercise
}

#[derive(Deserialize)]
pub struct Fever {
  pub days: UserInput<Days>,
  pub taken_temperature_today: UserInput<bool>,
  pub temperature_spot: UserInput<TemperatureSpot>,
  pub highest_temperature: UserInput<FarenheitTemperature>
}

#[derive(Deserialize)]
pub enum TemperatureSpot {
  Mouth, Ear, Armpit, Other(String)
}

// Temperature conversions are only for presentation, so in the apps
#[derive(Deserialize)]
pub struct FarenheitTemperature {
  pub value: f32
}

#[derive(Eq, PartialEq, Hash, Deserialize)]
pub enum SymptomId {
  Cough, Breathlessness, Fever, MuscleAches, LossSmellOrTaste, Diarrhea, RunnyNose, Other, None
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum UserInput<T> {
  Some(T),
  None
}

#[derive(Deserialize)]
pub struct EarliestSymptom {
  pub time: UserInput<UnixTime>
}

pub trait SymptomInputsSubmitter<
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> {
  fn submit_inputs(&self, inputs: SymptomInputs) -> Result<(), ServicesError>;
}

pub struct SymptomInputsSubmitterImpl<
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> {
  pub memo_mapper: MemoMapperType,
  pub tcn_keys: TcnKeysType,
  pub api: TcnApiType,
}

impl <
  MemoMapperType: MemoMapper, TcnKeysType: TcnKeys, TcnApiType: TcnApi
> SymptomInputsSubmitter<
  MemoMapperType, TcnKeysType, TcnApiType
> for SymptomInputsSubmitterImpl<
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

#[test]
fn test_inputs_to_report(){
  assert_eq!(1,1);

  let breathlessness = Breathlessness {
    cause: UserInput::Some(BreathlessnessCause::HurryOrHill)
  };

  let cough = Cough {
    cough_type: UserInput::Some(CoughType::Wet),
    days: UserInput::Some(Days {value: 3}),
    status: UserInput::Some(CoughStatus::SameOrSteadilyWorse),
  };

  let fever = Fever {
    days: UserInput::Some(Days { value: 2}),
    highest_temperature: UserInput::Some(FarenheitTemperature {value: 112.0}),
    taken_temperature_today: UserInput::Some(true),
    temperature_spot: UserInput::Some(TemperatureSpot::Armpit),
  };

  let earliest_symptom = EarliestSymptom {
    time: UserInput::Some(UnixTime{value: 1590356601})
  };

  let mut symptom_ids_set: HashSet<SymptomId> = HashSet::new();

  symptom_ids_set.insert(SymptomId::Cough);
  symptom_ids_set.insert(SymptomId::Fever);
  symptom_ids_set.insert(SymptomId::Diarrhea);



  let inputs : SymptomInputs = SymptomInputs {
    ids: symptom_ids_set,
    cough: cough,
    breathlessness: breathlessness,
    fever: fever,
    earliest_symptom: earliest_symptom,
  };


}