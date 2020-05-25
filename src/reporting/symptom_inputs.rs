use std::{io::Cursor, collections::HashSet};
use crate::{errors::ServicesError, reports_interval::UnixTime, tcn_ext::tcn_keys::TcnKeys, networking::TcnApi};
use serde::{Deserialize, Serialize};
use super::{memo::MemoMapper, public_report::PublicReport};
use tcn::SignedReport;

#[derive(Debug, Deserialize, Clone)]
pub struct SymptomInputs {
  pub ids: HashSet<SymptomId>,
  pub cough: Cough,
  pub breathlessness: Breathlessness,
  pub fever: Fever,
  pub earliest_symptom: EarliestSymptom
}

impl Default for SymptomInputs {
  fn default() -> Self { Self { 
    ids: HashSet::new(),
    cough: Cough {
      cough_type: UserInput::None,
      status: UserInput::None,
      days: UserInput::None,
    },
    breathlessness: Breathlessness {
      cause: UserInput::None,
    },
    fever: Fever {
      days: UserInput::None,
      temperature_spot: UserInput::None,
      taken_temperature_today: UserInput::None,
      highest_temperature: UserInput::None,
    },
    earliest_symptom: EarliestSymptom {
      time: UserInput::None,
    }
  }}
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cough {
  pub cough_type: UserInput<CoughType>,
  pub days: UserInput<Days>,
  pub status: UserInput<CoughStatus>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum CoughType {
  Wet, Dry
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum CoughStatus {
  BetterAndWorseThroughDay, WorseWhenOutside, SameOrSteadilyWorse
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Days {
  pub value: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Breathlessness {
  pub cause: UserInput<BreathlessnessCause>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum BreathlessnessCause {
  LeavingHouseOrDressing, WalkingYardsOrMinsOnGround, GroundOwnPace, HurryOrHill, Exercise
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Fever {
  pub days: UserInput<Days>,
  pub taken_temperature_today: UserInput<bool>,
  pub temperature_spot: UserInput<TemperatureSpot>,
  pub highest_temperature: UserInput<FarenheitTemperature>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TemperatureSpot {
  Mouth, Ear, Armpit, Other // Other(String)
}

// Temperature conversions are only for presentation, so in the apps
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FarenheitTemperature {
  pub value: f32
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize, Clone)]
pub enum SymptomId {
  Cough, Breathlessness, Fever, MuscleAches, LossSmellOrTaste, Diarrhea, RunnyNose, Other, None
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum UserInput<T> where T: Serialize {
  Some(T),
  None,
}

impl <T: Serialize> UserInput<T> {
  pub fn map<F: FnOnce(T) -> U, U: Serialize>(self, f: F) -> UserInput<U> {
    match self {
      UserInput::Some(input) => UserInput::Some(f(input)),
      UserInput::None => UserInput::None
    }
  }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
  pub memo_mapper: &'a MemoMapperType,
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
      println!("RUST Public report: {:?} doesn't contain infos relevant to other users. Not sending.", public_report);
      return Ok(())
    }

    println!("RUST Created public report: {:?}", public_report);

    let memo = self.memo_mapper.to_memo(public_report, UnixTime::now());

    println!("RUST mapped public report to memo: {:?}", memo.bytes);

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
fn test_public_report_with_inputs(){
  
  let breathlessness = Breathlessness {
    cause: UserInput::Some(BreathlessnessCause::HurryOrHill)
  };

  let cough = Cough {
    cough_type: UserInput::Some(CoughType::Dry),
    days: UserInput::Some(Days {value: 3}),
    status: UserInput::Some(CoughStatus::SameOrSteadilyWorse),
  };

  let fever = Fever {
    days: UserInput::Some(Days { value: 2}),
    highest_temperature: UserInput::Some(FarenheitTemperature {value: 100.5}),
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
  symptom_ids_set.insert(SymptomId::Breathlessness);

  let inputs : SymptomInputs = SymptomInputs {
    ids: symptom_ids_set,
    cough,
    breathlessness,
    fever,
    earliest_symptom,
  };

  let public_report = PublicReport::with_inputs(inputs);

  println!("{:#?}", public_report);
  /*
  PublicReport {
    earliest_symptom_time: Some(
        UnixTime {
            value: 1590356601,
        },
    ),
    fever_severity: Mild,
    cough_severity: Dry,
    breathlessness: true,
}
  */

  assert_eq!(CoughSeverity::Dry, public_report.cough_severity);
  assert_eq!(FeverSeverity::Mild, public_report.fever_severity);
  assert_eq!(true, public_report.breathlessness);

}

#[test]
fn test_public_report_should_be_sent(){
  assert_eq!(1, 1);

  let report_required_true = PublicReport {
    earliest_symptom_time: UserInput::Some(UnixTime{value: 1590356601}),
    fever_severity: FeverSeverity::Mild,
    cough_severity: CoughSeverity::Dry,
    breathlessness: true,
  };

  assert_eq!(true, report_required_true.should_be_sent());

  let report_required_false = PublicReport {
    earliest_symptom_time: UserInput::Some(UnixTime{value: 1590356601}),
    fever_severity: FeverSeverity::None,
    cough_severity: CoughSeverity::None,
    breathlessness: false,
  };

  assert_eq!(false, report_required_false.should_be_sent());

}