use crate::networking::{TcnApi, TcnApiImpl};
use crate::reports_updater::{
    ObservedTcnProcessor, ObservedTcnProcessorImpl, ReportsUpdater, TcnDao, TcnDaoImpl, TcnMatcher,
    TcnMatcherRayon,
};
use crate::{
    errors::ServicesError,
    init_persy,
    preferences::{Database, Preferences, PreferencesDao, PreferencesImpl},
    reporting::{
        memo::{MemoMapper, MemoMapperImpl},
        symptom_inputs::{SymptomInputs, SymptomInputsSubmitterImpl},
        symptom_inputs_manager::{
            SymptomInputsManagerImpl, SymptomInputsProcessor, SymptomInputsProcessorImpl,
        },
    },
    tcn_ext::tcn_keys::{TcnKeys, TcnKeysImpl},
};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rusqlite::Connection;
use std::sync::Arc;

#[allow(dead_code)]
pub struct CompositionRoot<'a, A, B, C, D, F, G, H, I>
where
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
    pub tcn_keys: Arc<I>,
}

pub static COMP_ROOT: OnceCell<
    CompositionRoot<
        PreferencesImpl,
        TcnDaoImpl,
        TcnMatcherRayon,
        TcnApiImpl,
        SymptomInputsProcessorImpl<
            SymptomInputsManagerImpl<
                SymptomInputsSubmitterImpl<
                    MemoMapperImpl,
                    TcnKeysImpl<PreferencesImpl>,
                    TcnApiImpl,
                >,
            >,
        >,
        ObservedTcnProcessorImpl<TcnDaoImpl>,
        MemoMapperImpl,
        TcnKeysImpl<PreferencesImpl>,
    >,
> = OnceCell::new();

pub fn bootstrap(db_path: &str) -> Result<(), ServicesError> {
    println!("Bootstrapping with db path: {:?}", db_path);

    // TODO should be in a dependency
    let persy_path = format!("{}/db.persy", db_path);
    init_persy(persy_path).map_err(ServicesError::from)?;

    let sqlite_path = format!("{}/db.sqlite", db_path);
    if let Err(_) = COMP_ROOT.set(create_comp_root(sqlite_path.as_ref())) {
        return Err(ServicesError::General(
            "Couldn't initialize dependencies".to_owned(),
        ));
    };

    Ok(())
}

pub fn dependencies() -> &'static CompositionRoot<
    'static,
    PreferencesImpl,
    TcnDaoImpl,
    TcnMatcherRayon,
    TcnApiImpl,
    SymptomInputsProcessorImpl<
        SymptomInputsManagerImpl<
            SymptomInputsSubmitterImpl<
                'static,
                MemoMapperImpl,
                TcnKeysImpl<PreferencesImpl>,
                TcnApiImpl,
            >,
        >,
    >,
    ObservedTcnProcessorImpl<'static, TcnDaoImpl>,
    MemoMapperImpl,
    TcnKeysImpl<PreferencesImpl>,
> {
    COMP_ROOT.get().expect("Not bootstrapped")
}

fn create_comp_root(
    sqlite_path: &str,
) -> CompositionRoot<
    'static,
    PreferencesImpl,
    TcnDaoImpl,
    TcnMatcherRayon,
    TcnApiImpl,
    SymptomInputsProcessorImpl<
        SymptomInputsManagerImpl<
            SymptomInputsSubmitterImpl<
                'static,
                MemoMapperImpl,
                TcnKeysImpl<PreferencesImpl>,
                TcnApiImpl,
            >,
        >,
    >,
    ObservedTcnProcessorImpl<'static, TcnDaoImpl>,
    MemoMapperImpl,
    TcnKeysImpl<PreferencesImpl>,
> {
    let api = &TcnApiImpl {};

    let database = Arc::new(Database::new(
        Connection::open(sqlite_path).expect("Couldn't create database!"),
    ));

    let preferences_dao = PreferencesDao::new(database);
    let preferences = Arc::new(PreferencesImpl {
        dao: preferences_dao,
    });

    let memo_mapper = &MemoMapperImpl {};

    let tcn_keys = Arc::new(TcnKeysImpl {
        preferences: preferences.clone(),
    });

    let symptom_inputs_submitter = SymptomInputsSubmitterImpl {
        memo_mapper,
        tcn_keys: tcn_keys.clone(),
        api,
    };

    let tcn_dao = &TcnDaoImpl {};

    CompositionRoot {
        api,
        reports_updater: ReportsUpdater {
            preferences: preferences.clone(),
            tcn_dao,
            tcn_matcher: TcnMatcherRayon {},
            api,
            memo_mapper,
        },
        symptom_inputs_processor: SymptomInputsProcessorImpl {
            inputs_manager: SymptomInputsManagerImpl {
                inputs: Arc::new(RwLock::new(SymptomInputs::default())),
                inputs_submitter: symptom_inputs_submitter,
            },
        },
        observed_tcn_processor: ObservedTcnProcessorImpl { tcn_dao },
        tcn_keys: tcn_keys.clone(),
    }
}
