use super::symptom_inputs::{Cough, CoughType, Fever, SymptomId, SymptomInputs, UserInput};
use crate::reports_interval::UnixTime;
use log::info;
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
    pub muscle_aches: bool,
    pub loss_smell_or_taste: bool,
    pub diarrhea: bool,
    pub runny_nose: bool,
    pub other: bool,
}

impl PublicReport {
    pub fn with_inputs(inputs: SymptomInputs, report_time: UnixTime) -> Option<PublicReport> {
        let earliest_symptom_time = inputs.earliest_symptom.time.clone();
        let fever_severity = to_fever_severity(&inputs.fever);
        let cough_severity =
            to_cough_severity(&inputs.cough, inputs.ids.contains(&SymptomId::Cough));
        let breathlessness = inputs.ids.contains(&SymptomId::Breathlessness).clone();
        let muscle_aches = inputs.ids.contains(&SymptomId::MuscleAches).clone();
        let loss_smell_or_taste = inputs.ids.contains(&SymptomId::LossSmellOrTaste).clone();
        let diarrhea = inputs.ids.contains(&SymptomId::Diarrhea).clone();
        let runny_nose = inputs.ids.contains(&SymptomId::RunnyNose).clone();
        let other = inputs.ids.contains(&SymptomId::Other).clone();

        if fever_severity != FeverSeverity::None
            || cough_severity != CoughSeverity::None
            || breathlessness
            || muscle_aches
            || loss_smell_or_taste
            || diarrhea
            || runny_nose
            || other
        {
            Some(PublicReport {
                report_time,
                earliest_symptom_time,
                fever_severity,
                cough_severity,
                breathlessness,
                muscle_aches,
                loss_smell_or_taste,
                diarrhea,
                runny_nose,
                other,
            })
        } else {
            info!(
                "Inputs: {:?} don't contain infos relevant to other users. Public report not generated.",
                inputs
            );
            None
        }
    }
}

fn to_fever_severity(fever: &Fever) -> FeverSeverity {
    match &fever.highest_temperature {
        UserInput::None => FeverSeverity::None,
        UserInput::Some(temp) => match temp.value {
            t if t > 100.6 => FeverSeverity::Serious,
            t if t > 98.6 => FeverSeverity::Mild,
            _ => FeverSeverity::None,
        },
    }
}

fn to_cough_severity(cough: &Cough, selected_has_cough: bool) -> CoughSeverity {
    match &cough.cough_type {
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
