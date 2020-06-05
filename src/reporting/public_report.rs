use super::symptom_inputs::{Cough, CoughType, Fever, SymptomId, SymptomInputs, UserInput};
use crate::reports_interval::UnixTime;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum FeverSeverity {
    None,
    Mild,
    Serious,
}
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum CoughSeverity {
    None,
    Existing,
    Wet,
    Dry,
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PublicReport {
    pub report_time: UnixTime,
    pub earliest_symptom_time: UserInput<UnixTime>,
    pub fever_severity: FeverSeverity,
    pub cough_severity: CoughSeverity,
    pub breathlessness: bool,
}

impl PublicReport {
    pub fn should_be_sent(&self) -> bool {
        self.fever_severity != FeverSeverity::None
            || self.cough_severity != CoughSeverity::None
            || self.breathlessness
    }

    pub fn with_inputs(inputs: SymptomInputs, report_time: UnixTime) -> PublicReport {
        PublicReport {
            report_time,
            breathlessness: inputs.ids.contains(&SymptomId::Breathlessness),
            earliest_symptom_time: inputs.earliest_symptom.time,
            fever_severity: to_fever_severity(inputs.fever),
            cough_severity: to_cough_severity(inputs.cough, inputs.ids.contains(&SymptomId::Cough)),
        }
    }
}

fn to_fever_severity(fever: Fever) -> FeverSeverity {
    match fever.highest_temperature {
        UserInput::None => FeverSeverity::None,
        UserInput::Some(temp) => match temp.value {
            t if t > 100.6 => FeverSeverity::Serious,
            t if t > 98.6 => FeverSeverity::Mild,
            _ => FeverSeverity::None,
        },
    }
}

fn to_cough_severity(cough: Cough, selected_has_cough: bool) -> CoughSeverity {
    match cough.cough_type {
        UserInput::None => {
            if selected_has_cough {
                CoughSeverity::Existing
            } else {
                CoughSeverity::None
            }
        }
        UserInput::Some(cough_type) => match cough_type {
            CoughType::Wet => CoughSeverity::Wet,
            CoughType::Dry => CoughSeverity::Dry,
        },
    }
}
