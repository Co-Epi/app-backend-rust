use crate::networking::{TcnApiImpl, TcnApi};

pub struct CompositionRoot<T: TcnApi> {
  pub api: T,
  // reports_updater: ReportsUpdater
}

pub static COMP_ROOT: CompositionRoot<TcnApiImpl> = CompositionRoot { 
  api: TcnApiImpl {}
};
