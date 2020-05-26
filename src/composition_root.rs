use crate::networking::{TcnApiImpl, TcnApi};
use crate::reports_updater::{TcnMatcher, ReportsUpdater, TcnDao, TcnDaoImpl, TcnMatcherImpl, ObservedTcnProcessor, ObservedTcnProcessorImpl};
use crate::{reporting::{memo::MemoMapperImpl, symptom_inputs::{SymptomInputsSubmitterImpl, SymptomInputsSubmitter, SymptomInputs}, symptom_inputs_manager::{SymptomInputsManagerImpl, SymptomInputsProcessorImpl, SymptomInputsProcessor}}, preferences::{Preferences, PreferencesImpl}, tcn_ext::tcn_keys::TcnKeysImpl};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct CompositionRoot<'a, A, B, C, D, E, F, G> where 
  A: Preferences,
  B: TcnDao,
  C: TcnMatcher,
  D: TcnApi,
  E: SymptomInputsSubmitter<MemoMapperImpl, TcnKeysImpl<A>, D>, // TODO no concrete types here?
  F: SymptomInputsProcessor,
  G: ObservedTcnProcessor,
{
  pub api: &'a D,
  pub reports_updater: ReportsUpdater<'a, A, B, C, D>,
  pub symptom_inputs_submitter: E,
  pub symptom_inputs_processor: F,
  pub observed_tcn_processor: G,
}

pub static COMP_ROOT: Lazy<
  CompositionRoot<
    PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl, 
    SymptomInputsSubmitterImpl<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>,
    SymptomInputsProcessorImpl<SymptomInputsManagerImpl<SymptomInputsSubmitterImpl<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>>>,
    ObservedTcnProcessorImpl<TcnDaoImpl>
  >
> = 
  Lazy::new(|| create_comp_root());

fn create_comp_root() -> CompositionRoot<'static, 
  PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl, 
  SymptomInputsSubmitterImpl<'static, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>,
  SymptomInputsProcessorImpl<SymptomInputsManagerImpl<SymptomInputsSubmitterImpl<'static, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>>>,
  ObservedTcnProcessorImpl<'static, TcnDaoImpl>
> {
  let api = &TcnApiImpl {};
  let preferences = Arc::new(PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) });
  let symptom_inputs_submitter = SymptomInputsSubmitterImpl { 
    memo_mapper: MemoMapperImpl {},  
    tcn_keys: TcnKeysImpl { 
      preferences: preferences.clone()
    },
    api
  };

  let tcn_dao = &TcnDaoImpl {};

  CompositionRoot { 
    api: api,
    reports_updater: ReportsUpdater { 
      preferences: preferences.clone(),
      tcn_dao,
      tcn_matcher: TcnMatcherImpl {},
      api
    },
    symptom_inputs_submitter: SymptomInputsSubmitterImpl { 
      memo_mapper: MemoMapperImpl {},  
      tcn_keys: TcnKeysImpl { 
        preferences: preferences.clone()
      },
      api
    },
    symptom_inputs_processor: SymptomInputsProcessorImpl {
      inputs_manager: SymptomInputsManagerImpl {
        inputs: Arc::new(RwLock::new(SymptomInputs::default())),
        inputs_submitter: symptom_inputs_submitter
      }
    },
    observed_tcn_processor: ObservedTcnProcessorImpl {
      tcn_dao
    }
  }
}
