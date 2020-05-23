
use crate::reports_interval::UnixTime;

#[derive(Debug, PartialEq, Clone)]
pub enum UserInput<T> {
  Some(T),
  None
}

#[derive(Debug, PartialEq, Clone)]
pub enum FeverSeverity {
  None, Mild, Serious
}
#[derive(Debug, PartialEq, Clone)]
pub enum CoughSeverity {
  None, Existing, Wet, Dry
}

#[derive(Debug, PartialEq, Clone)]
pub struct PublicReport {
  pub earliest_symptom_time: UserInput<UnixTime>,
  pub fever_severity: FeverSeverity,
  pub cough_severity: CoughSeverity,
  pub breathlessness: bool
}

impl PublicReport {
  fn should_be_sent(&self) -> bool {
    self.fever_severity != FeverSeverity::None || self.cough_severity != CoughSeverity::None || self.breathlessness
  }
}
