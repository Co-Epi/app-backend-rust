use crate::networking::{TcnApiImpl, TcnApi};
use crate::reports_updater::{TcnMatcher, ReportsUpdater, TcnDao, TcnDaoImpl, TcnMatcherImpl};
use crate::{reporting::{memo::{MemoMapperImpl}, symptom_inputs::{SymptomInputsSubmitterImpl, SymptomInputsSubmitter}}, preferences::{Preferences, PreferencesImpl}, tcn_ext::tcn_keys::{TcnKeysImpl, TcnKeys}};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

pub struct CompositionRoot<'a,
  PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi, 
  // TODO don't pass concrete type for MemoMapper / TcnKeys here?
  SymptomInputsSubmitterType: SymptomInputsSubmitter<MemoMapperImpl, TcnKeysImpl<PreferencesType>, ApiType>, 
> {
  pub api: ApiType,
  pub reports_updater: ReportsUpdater<'a, PreferencesType, TcnDaoType, TcnMatcherType, ApiType>,
  pub symptom_inputs_submitter: SymptomInputsSubmitterType
}

pub static COMP_ROOT: Lazy<
  CompositionRoot<
    PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl, 
    SymptomInputsSubmitterImpl<MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>
  >
> = 
  Lazy::new(|| create_comp_root());

fn create_comp_root() -> CompositionRoot<'static, 
  PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl, 
  SymptomInputsSubmitterImpl<'static, MemoMapperImpl, TcnKeysImpl<PreferencesImpl>, TcnApiImpl>
> {
  // FIXME pass the same instances / references
  let api = &TcnApiImpl {};
  // let preferences = PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) };

  CompositionRoot { 
    api: TcnApiImpl {},
    reports_updater: ReportsUpdater { 
      preferences: PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) },
      tcn_dao: TcnDaoImpl {},
      tcn_matcher: TcnMatcherImpl {},
      api
    },
    symptom_inputs_submitter: SymptomInputsSubmitterImpl { 
      memo_mapper: MemoMapperImpl {},  
      tcn_keys: TcnKeysImpl { 
        preferences: PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) }
      },
      api
    }
  }
}
