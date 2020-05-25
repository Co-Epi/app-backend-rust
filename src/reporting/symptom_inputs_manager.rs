
use std::{collections::HashSet, sync::Arc};
use super::{memo::MemoMapperImpl, symptom_inputs::{UserInput, SymptomId, Days, CoughType, CoughStatus, BreathlessnessCause, FarenheitTemperature, SymptomInputs, TemperatureSpot, SymptomInputsSubmitter}};
use parking_lot::RwLock;
use crate::{reports_interval::UnixTime, errors::ServicesError, tcn_ext::tcn_keys::TcnKeysImpl, networking::TcnApiImpl, preferences::PreferencesImpl};
use chrono::{Duration, Utc};

pub trait SymptomInputsProcessor {
  fn set_symptom_ids(&self, ids: &str) -> Result<(), ServicesError>;
  fn set_cough_type(&self, cough_type: &str) -> Result<(), ServicesError>;
  fn set_cough_days(&self, is_set: bool, days: u32) -> Result<(), ServicesError>;
  fn set_cough_status(&self, status: &str) -> Result<(), ServicesError>;
  fn set_breathlessness_cause(&self, cause: &str) -> Result<(), ServicesError>;
  fn set_fever_days(&self, is_set: bool, days: u32) -> Result<(), ServicesError>;
  fn set_fever_taken_temperature_today(&self, is_set: bool, taken: bool) -> Result<(), ServicesError>;
  fn set_fever_taken_temperature_spot(&self, spot: &str) -> Result<(), ServicesError>;
  fn set_fever_highest_temperature_taken(&self, is_set: bool, temperature: f32) -> Result<(), ServicesError>;
  fn set_earliest_symptom_started_days_ago(&self, is_set: bool, days: u32) -> Result<(), ServicesError>;
  
  fn submit(&self) -> Result<(), ServicesError>;
  fn clear(&self) -> Result<(), ServicesError>;
}

pub struct SymptomInputsProcessorImpl<A> where A: SymptomInputsManager {
  pub inputs_manager: A
}

impl <A> SymptomInputsProcessor for SymptomInputsProcessorImpl<A> where A: SymptomInputsManager {

    fn set_symptom_ids(&self, ids: &str) -> Result<(), ServicesError> {
      let inputs: Vec<&str> = serde_json::from_str(ids)?;

      let mut symptom_ids = HashSet::new();
      for str_id in inputs {
        let symptom_id = match str_id {
          "cough" => SymptomId::Cough,
          "breathlessness" => SymptomId::Breathlessness,
          "fever" =>  SymptomId::Fever,
          "muscle_aches" =>  SymptomId::MuscleAches,
          "loss_smell_or_taste" => SymptomId::LossSmellOrTaste,
          "diarrhea" =>  SymptomId::Diarrhea,
          "runny_nose" => SymptomId::RunnyNose,
          "other" =>  SymptomId::Other,
          "none" => SymptomId::None,
          _ => Err(format!("RUST Not supported: {}", str_id))?
        };
        symptom_ids.insert(symptom_id);
      }

      self.inputs_manager.select_symptom_ids(symptom_ids);

      Ok(())
    }

    fn set_cough_type(&self, cough_type: &str) -> Result<(), ServicesError> {
      let input = match cough_type {
        "none" => UserInput::None,
        "wet" => UserInput::Some(CoughType::Wet),
        "dry" => UserInput::Some(CoughType::Dry),
        _ => Err(format!("Not supported: {}", cough_type))?
      };

      println!("RUST setting cough type: {:?}", input);

      self.inputs_manager.set_cough_type(input);
      Ok(())
    }

    fn set_cough_days(&self, is_set: bool, days: u32) -> Result<(), ServicesError> {
      let input = match is_set {
        true => UserInput::Some(Days { value: days }),
        false => UserInput::None
      };

      println!("RUST setting cough days {:?}", input);
 
      self.inputs_manager.set_cough_days(input);
      Ok(())
    }

    fn set_cough_status(&self, status: &str) -> Result<(), ServicesError> {
      let input = match status {
        "none" => UserInput::None,
        "better_and_worse" => UserInput::Some(CoughStatus::BetterAndWorseThroughDay),
        "same_steadily_worse" => UserInput::Some(CoughStatus::SameOrSteadilyWorse),
        "worse_outside" => UserInput::Some(CoughStatus::WorseWhenOutside),
        _ => Err(format!("Not supported: {}", status))?
      };

      println!("RUST setting cough status: {:?}", input);

      self.inputs_manager.set_cough_status(input);
      Ok(())
    }

    fn set_breathlessness_cause(&self, cause: &str) -> Result<(), ServicesError> {
      let input = match cause {
        "none" => UserInput::None,
        "exercise" => UserInput::Some(BreathlessnessCause::Exercise),
        "leaving_house_or_dressing" => UserInput::Some(BreathlessnessCause::LeavingHouseOrDressing),
        "walking_yards_or_mins_on_ground" => UserInput::Some(BreathlessnessCause::WalkingYardsOrMinsOnGround),
        "ground_own_pace" => UserInput::Some(BreathlessnessCause::GroundOwnPace),
        "hurry_or_hill" => UserInput::Some(BreathlessnessCause::HurryOrHill),
        _ => Err(format!("Not supported: {}", cause))?
      };

      println!("RUST setting breathlessness cause: {:?}", input);

      self.inputs_manager.set_breathlessness_cause(input);
      Ok(()) 
    }

    fn set_fever_days(&self, is_set: bool, days: u32) -> Result<(), ServicesError> {
      let input = match is_set {
        true => UserInput::Some(Days { value: days }),
        false => UserInput::None
      };

      println!("RUST setting fever days {:?}", input);
 
      self.inputs_manager.set_fever_days(input);
      Ok(()) 
    }

    fn set_fever_taken_temperature_today(&self, is_set: bool, taken: bool) -> Result<(), ServicesError> {
      let input = match is_set {
        true => UserInput::Some(taken),
        false => UserInput::None
      };

      println!("RUST setting taken temperature today {:?}", input);
 
      self.inputs_manager.set_fever_taken_temperature_today(input);
      Ok(())  
    }

    fn set_fever_taken_temperature_spot(&self, spot: &str) -> Result<(), ServicesError> {
      let input = match spot {
        "none" => UserInput::None,
        "armpit" => UserInput::Some(TemperatureSpot::Armpit),
        "ear" => UserInput::Some(TemperatureSpot::Ear),
        "mouth" => UserInput::Some(TemperatureSpot::Mouth),
        "other" => UserInput::Some(TemperatureSpot::Other),
        _ => Err(format!("Not supported: {}", spot))?
      };

      println!("RUST setting fever temperature spot: {:?}", input);

      self.inputs_manager.set_fever_taken_temperature_spot(input);
      Ok(()) 
    }

    fn set_fever_highest_temperature_taken(&self, is_set: bool, temperature: f32) -> Result<(), ServicesError> {
      let input = match is_set {
        true => UserInput::Some(FarenheitTemperature { value: temperature }),
        false => UserInput::None
      };

      println!("RUST setting highest temperature taken {:?}", input);
 
      self.inputs_manager.set_fever_highest_temperature_taken(input);
      Ok(()) 
    }

    fn set_earliest_symptom_started_days_ago(&self, is_set: bool, days: u32) -> Result<(), ServicesError> {
      let input = match is_set {
        true => UserInput::Some(Days { value: days }),
        false => UserInput::None
      };

      println!("RUST setting earliest symptom days ago {:?}", input);
 
      self.inputs_manager.set_earliest_symptom_started_days_ago(input);
      Ok(()) 
    }

    fn submit(&self) -> Result<(), ServicesError> {
      self.inputs_manager.submit()
    }

    fn clear(&self) -> Result<(), ServicesError> {
      self.inputs_manager.clear();
      Ok(())
    }
}

pub trait SymptomInputsManager {
    fn select_symptom_ids(&self, ids: HashSet<SymptomId>);
    fn set_cough_type(&self, input: UserInput<CoughType>);
    fn set_cough_days(&self, input: UserInput<Days>);
    fn set_cough_status(&self, status: UserInput<CoughStatus>);
    fn set_breathlessness_cause(&self, cause: UserInput<BreathlessnessCause>);
    fn set_fever_days(&self, days: UserInput<Days>);
    fn set_fever_taken_temperature_today(&self, taken: UserInput<bool>);
    fn set_fever_taken_temperature_spot(&self, spot: UserInput<TemperatureSpot>);
    fn set_fever_highest_temperature_taken(&self, temp: UserInput<FarenheitTemperature>);
    fn set_earliest_symptom_started_days_ago(&self, days: UserInput<Days>);
    
    fn submit(&self) -> Result<(), ServicesError>;
    fn clear(&self);
}

pub struct SymptomInputsManagerImpl<A> where 
  // TODO no concrete types here?
  A: SymptomInputsSubmitter<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>
{
  pub inputs: Arc<RwLock<SymptomInputs>>,
  pub inputs_submitter: A
}

impl <A> SymptomInputsManagerImpl<A> where
  // TODO no concrete types here?
  A: SymptomInputsSubmitter<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>
{
  fn print_current_state(&self) {
    println!("RUST symptom inputs state: {:?}", self.inputs);
  }
}

impl <A> SymptomInputsManager for SymptomInputsManagerImpl<A> where
  // TODO no concrete types here?
  A: SymptomInputsSubmitter<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>
{

    fn select_symptom_ids(&self, ids: HashSet<SymptomId>) {
      self.inputs.write().ids = ids;
      self.print_current_state();
    }

    fn set_cough_type(&self, input: UserInput<CoughType>) {
      self.inputs.write().cough.cough_type = input;
      self.print_current_state();
    }

    fn set_cough_days(&self, input: UserInput<Days>) {
      self.inputs.write().cough.days = input;
      self.print_current_state();
    }

    fn set_cough_status(&self, input: UserInput<CoughStatus>) {
      self.inputs.write().cough.status = input;
      self.print_current_state();
    }

    fn set_breathlessness_cause(&self, input: UserInput<BreathlessnessCause>) {
      self.inputs.write().breathlessness.cause = input;
      self.print_current_state();
    }

    fn set_fever_days(&self, input: UserInput<Days>) {
      self.inputs.write().fever.days = input;
      self.print_current_state();
    }

    fn set_fever_taken_temperature_today(&self, input: UserInput<bool>) {
      self.inputs.write().fever.taken_temperature_today = input;
      self.print_current_state();
    }

    fn set_fever_taken_temperature_spot(&self, input: UserInput<TemperatureSpot>) {
      self.inputs.write().fever.temperature_spot = input;
      self.print_current_state();
    }

    fn set_fever_highest_temperature_taken(&self, input: UserInput<FarenheitTemperature>) {
      self.inputs.write().fever.highest_temperature = input;
      self.print_current_state();
    }

    fn set_earliest_symptom_started_days_ago(&self, input: UserInput<Days>) {
      let time = input.map (|days| {
        let date_time = Utc::now() - Duration::days(days.value as i64);
        UnixTime { value: date_time.timestamp() as u64 }
      });

      self.inputs.write().earliest_symptom.time = time;
      self.print_current_state();
    }

    fn submit(&self) -> Result<(), ServicesError> {
      println!("RUST Submitting symptom inputs...");
      self.print_current_state();
      let result = self.inputs_submitter.submit_inputs(self.inputs.read().clone());

      if result.is_ok() {
        self.clear()
      }
      // TODO: if submit doesn't succeed, when to clear the inputs? 

      result
    }

    fn clear(&self) {
      // replace SymptomInputs instance with default or reset fields individually 
      println!("RUST TODO implement clear symptoms");
      self.print_current_state();
    }
}