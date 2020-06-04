use crate::networking::{TcnApiImpl, TcnApi};
use crate::reports_updater::{TcnMatcher, ReportsUpdater, TcnDao, TcnDaoImpl, TcnMatcherRayon, ObservedTcnProcessor, ObservedTcnProcessorImpl};
use crate::{reporting::{memo::{MemoMapper, MemoMapperImpl}, symptom_inputs::{SymptomInputsSubmitterImpl, SymptomInputs}, symptom_inputs_manager::{SymptomInputsManagerImpl, SymptomInputsProcessorImpl, SymptomInputsProcessor}}, preferences::{Preferences, PreferencesImpl}, tcn_ext::tcn_keys::{TcnKeys, TcnKeysImpl}};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

#[allow(dead_code)]
pub struct CompositionRoot<'a, A, B, C, D, F, G, H, I> where 
  A: Preferences,
  B: TcnDao,
  C: TcnMatcher,
  D: TcnApi,
  F: SymptomInputsProcessor,
  G: ObservedTcnProcessor,
  H: MemoMapper,
  I: TcnKeys,
{
  pub api: &'a D,
  pub reports_updater: ReportsUpdater<'a, A, B, C, D, H>,
  pub symptom_inputs_processor: F,
  pub observed_tcn_processor: G,
  pub tcn_keys: Arc<I>
}

pub static COMP_ROOT: Lazy<
  CompositionRoot<
    PreferencesImpl, TcnDaoImpl, TcnMatcherRayon, TcnApiImpl, 
    SymptomInputsProcessorImpl<SymptomInputsManagerImpl<SymptomInputsSubmitterImpl<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>>>,
    ObservedTcnProcessorImpl<TcnDaoImpl>, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>
  >
> = 
  Lazy::new(|| create_comp_root());


fn create_comp_root() -> CompositionRoot<'static, 
  PreferencesImpl, TcnDaoImpl, TcnMatcherRayon, TcnApiImpl, 
  SymptomInputsProcessorImpl<SymptomInputsManagerImpl<SymptomInputsSubmitterImpl<'static, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>>>,
  ObservedTcnProcessorImpl<'static, TcnDaoImpl>, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>
> {
  let api = &TcnApiImpl {};
  let preferences = Arc::new(PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) });
  let memo_mapper = &MemoMapperImpl {};

  let tcn_keys = Arc::new(TcnKeysImpl { 
    preferences: preferences.clone()
  });

  let symptom_inputs_submitter = SymptomInputsSubmitterImpl { 
    memo_mapper,  
    tcn_keys: tcn_keys.clone(),
    api
  };

  let tcn_dao = &TcnDaoImpl {};

  CompositionRoot { 
    api,
    reports_updater: ReportsUpdater { 
      preferences: preferences.clone(),
      tcn_dao,
      tcn_matcher: TcnMatcherRayon {},
      api,
      memo_mapper
    },
    symptom_inputs_processor: SymptomInputsProcessorImpl {
      inputs_manager: SymptomInputsManagerImpl {
        inputs: Arc::new(RwLock::new(SymptomInputs::default())),
        inputs_submitter: symptom_inputs_submitter
      }
    },
    observed_tcn_processor: ObservedTcnProcessorImpl {
      tcn_dao
    },
    tcn_keys: tcn_keys.clone()
  }
}
