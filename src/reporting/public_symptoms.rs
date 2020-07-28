use super::symptom_inputs::{Cough, CoughType, Fever, SymptomId, SymptomInputs, UserInput};
use crate::{errors::ServicesError, reports_interval::UnixTime};
use log::info;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Serialize, Eq)]
pub enum FeverSeverity {
    None,
    Mild,
    Serious,
}

impl FeverSeverity {
    pub fn raw_value(&self) -> u8 {
        match self {
            FeverSeverity::None => 0,
            FeverSeverity::Mild => 1,
            FeverSeverity::Serious => 2,
        }
    }

    pub fn from(raw_value: u8) -> Result<FeverSeverity, ServicesError> {
        match raw_value {
            0 => Ok(FeverSeverity::None),
            1 => Ok(FeverSeverity::Mild),
            2 => Ok(FeverSeverity::Serious),
            _ => Err(ServicesError::General(format!(
                "Not supported: {}",
                raw_value
            ))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Eq)]
pub enum CoughSeverity {
    None,
    Existing,
    Wet,
    Dry,
}

impl CoughSeverity {
    pub fn raw_value(&self) -> u8 {
        match self {
            CoughSeverity::None => 0,
            CoughSeverity::Existing => 1,
            CoughSeverity::Dry => 2,
            CoughSeverity::Wet => 3,
        }
    }

    pub fn from(raw_value: u8) -> Result<CoughSeverity, ServicesError> {
        match raw_value {
            0 => Ok(CoughSeverity::None),
            1 => Ok(CoughSeverity::Existing),
            2 => Ok(CoughSeverity::Dry),
            3 => Ok(CoughSeverity::Wet),
            _ => Err(ServicesError::General(format!(
                "Not supported: {}",
                raw_value
            ))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Eq)]
pub struct PublicSymptoms {
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
    pub no_symptoms: bool, // https://github.com/Co-Epi/app-ios/issues/268#issuecomment-645583717
}

impl PublicSymptoms {
    pub fn with_inputs(inputs: SymptomInputs, report_time: UnixTime) -> Option<PublicSymptoms> {
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
        let no_symptoms = inputs.ids.contains(&SymptomId::None).clone();

        if fever_severity != FeverSeverity::None
            || cough_severity != CoughSeverity::None
            || breathlessness
            || muscle_aches
            || loss_smell_or_taste
            || diarrhea
            || runny_nose
            || other
            || no_symptoms
        {
            Some(PublicSymptoms {
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
                no_symptoms,
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
