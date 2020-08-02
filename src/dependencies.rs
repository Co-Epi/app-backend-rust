use crate::networking::{TcnApi, TcnApiImpl};
use crate::{
    database::{
        alert_dao::{AlertDao, AlertDaoImpl},
        database::Database,
        migration::Migration,
        preferences::{Preferences, PreferencesDao, PreferencesImpl},
        tcn_dao::{TcnDao, TcnDaoImpl},
    },
    errors::ServicesError,
    expect_log,
    reporting::{
        memo::{MemoMapper, MemoMapperImpl},
        symptom_inputs::{SymptomInputs, SymptomInputsSubmitterImpl},
        symptom_inputs_manager::{
            SymptomInputsManagerImpl, SymptomInputsProcessor, SymptomInputsProcessorImpl,
        },
    },
    reports_update::{
        exposure::ExposureGrouper,
        reports_updater::ReportsUpdater,
        tcn_matcher::{TcnMatcher, TcnMatcherRayon},
    },
    tcn_ext::tcn_keys::{TcnKeys, TcnKeysImpl},
    tcn_recording::{
        observed_tcn_processor::{ObservedTcnProcessor, ObservedTcnProcessorImpl},
        tcn_batches_manager::TcnBatchesManager,
    },
};
use log::*;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rusqlite::Connection;
use std::sync::Arc;

#[allow(dead_code)]
pub struct Dependencies<'a, A, B, C, D, F, G, H, I, J>
where
    A: Preferences,
    B: TcnDao,
    C: TcnMatcher,
    D: TcnApi,
    F: SymptomInputsProcessor,
    G: ObservedTcnProcessor,
    H: MemoMapper,
    I: TcnKeys,
    J: AlertDao,
{
    pub api: &'a D,
    pub reports_updater: ReportsUpdater<'a, A, B, C, D, H, J>,
    pub symptom_inputs_processor: F,
    pub observed_tcn_processor: G,
    pub tcn_keys: Arc<I>,
    pub alert_dao: Arc<J>,
}

pub static DEPENDENCIES: OnceCell<
    Dependencies<
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
        AlertDaoImpl,
    >,
> = OnceCell::new();

pub fn bootstrap(db_path: &str) -> Result<(), ServicesError> {
    info!("Bootstrapping with db path: {:?}", db_path);

    let sqlite_path = format!("{}/db.sqlite", db_path);
    debug!("Sqlite path: {:?}", sqlite_path);

    if let Err(_) = DEPENDENCIES.set(create_dependencies(sqlite_path.as_ref())) {
        return Err(ServicesError::General(
            "Couldn't initialize dependencies".to_owned(),
        ));
    };

    Ok(())
}

pub fn dependencies() -> &'static Dependencies<
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
    ObservedTcnProcessorImpl<TcnDaoImpl>,
    MemoMapperImpl,
    TcnKeysImpl<PreferencesImpl>,
    AlertDaoImpl,
> {
    let res = DEPENDENCIES
        .get()
        .ok_or(ServicesError::General("DEPENDENCIES not set".to_owned()));

    // Note that the error message here is unlikely to appear on Android, as if DEPENDENCIES is not set
    // most likely bootstrap hasn't been executed (which initializes the logger)
    expect_log!(
        res,
        "DEPENDENCIES not set. Maybe app didn't call bootstrap?"
    )
}

fn create_dependencies(
    sqlite_path: &str,
) -> Dependencies<
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
    ObservedTcnProcessorImpl<TcnDaoImpl>,
    MemoMapperImpl,
    TcnKeysImpl<PreferencesImpl>,
    AlertDaoImpl,
> {
    let api = &TcnApiImpl {};

    let connection_res = Connection::open(sqlite_path);
    let connection = expect_log!(connection_res, "Couldn't create database!");
    let database = Arc::new(Database::new(connection));

    let migration_handler = Migration::new(database.clone());
    migration_handler.run_db_migrations();

    let preferences_dao = PreferencesDao::new(database.clone());
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

    let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));
    let alert_dao = Arc::new(AlertDaoImpl::new(database));

    let exposure_grouper = ExposureGrouper { threshold: 3600 };

    Dependencies {
        api,
        reports_updater: ReportsUpdater {
            preferences: preferences.clone(),
            tcn_dao: tcn_dao.clone(),
            tcn_matcher: TcnMatcherRayon {},
            api,
            memo_mapper,
            exposure_grouper: exposure_grouper.clone(),
            alert_dao: alert_dao.clone(),
        },
        symptom_inputs_processor: SymptomInputsProcessorImpl {
            inputs_manager: SymptomInputsManagerImpl {
                inputs: Arc::new(RwLock::new(SymptomInputs::default())),
                inputs_submitter: symptom_inputs_submitter,
            },
        },
        observed_tcn_processor: ObservedTcnProcessorImpl::new(TcnBatchesManager::new(
            tcn_dao.clone(),
            exposure_grouper,
        )),
        tcn_keys,
        alert_dao,
    }
}
