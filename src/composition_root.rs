use crate::networking::{TcnApiImpl, TcnApi};
use crate::reports_updater::{TcnMatcher, ReportsUpdater, TcnDao, Preferences, PreferencesImpl, TcnDaoImpl, TcnMatcherImpl};
use once_cell::sync::Lazy;
use parking_lot::RwLock;

pub struct CompositionRoot<
PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi> {
  pub api: ApiType,
  pub reports_updater: ReportsUpdater<PreferencesType, TcnDaoType, TcnMatcherType, ApiType>
}

pub static COMP_ROOT: Lazy<CompositionRoot<PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl>> = 
  Lazy::new(|| create_comp_root());

fn create_comp_root() -> CompositionRoot<PreferencesImpl, TcnDaoImpl, TcnMatcherImpl, TcnApiImpl> {
  let api = TcnApiImpl {};
  let preferences = PreferencesImpl { config: RwLock::new(confy::load("coepi").unwrap()) };

  CompositionRoot { 
    api: TcnApiImpl {},
    reports_updater: ReportsUpdater { 
      preferences,
      tcn_dao: TcnDaoImpl {},
      tcn_matcher: TcnMatcherImpl {},
      api
    }
  }
}
