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

    let connection_res = Connection::open(sqlite_path);
    let connection = expect_log!(connection_res, "Couldn't create database!");
    let database = Arc::new(Database::new(connection));

    if let Err(_) = DEPENDENCIES.set(create_dependencies(database, 1)) {
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
    database: Arc<Database>,
    required_db_version: i32,
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
    let migration_handler = Migration::new(database.clone());
    migration_handler.run_db_migrations(required_db_version);

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
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Row};


    #[test]
    fn test_create_dependencies_with_migration_from_03_to_04(){
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));

        prep_data_03(database.clone());
        let pragma_variable_name = "user_version";
        let db_version: i32 = database.core_pragma_query(pragma_variable_name);

        assert_eq!(0, db_version);

        create_dependencies(database.clone(), 3);

        let new_db_version: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(1, new_db_version);

        let columns_6 = core_table_info("tcn", database.clone());
        assert_eq!(6, columns_6.len());

    }

    fn prep_data_03(database: Arc<Database>){

        let exported_db_sql = "BEGIN TRANSACTION;
        CREATE TABLE IF NOT EXISTS tcn(
                        tcn text not null,
                        contact_time integer not null
                    );
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('f3c939d7741f4a9be1c3c44dae084e7a',1595240743);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('4d621482b4aff1a6680d46a589269fd3',1596387734);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('188c9bfc1e675c7e0797cc43a015a60d',1596387735);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('c65a443a6563ad2d328ae8594f96b27b',1596387741);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('67347e90140555affb4c59795febbdde',1596387991);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('244a3961eb0e8407346ad525f16172ff',1596388633);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('39c195bd27dae245577f03dd5c48f244',1596388633);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('30afb71fd9db5dea604c52cc11969c54',1596388638);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('b2c4247d156106ccae799e530df63d61',1596388645);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('c76101bb7831e8a15d9e54978660a801',1596389536);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('264592dc4f280a31923cbe1f178ee16f',1596389537);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('e3f4b9bad40de7bbb91af599196cc07c',1596389539);
        INSERT INTO `tcn` (tcn,contact_time) VALUES ('c32b29785387807c13edc8ac3c5b030e',1596389539);
        CREATE TABLE IF NOT EXISTS preferences(
                        key text primary key,
                        value text not null
                    );
        INSERT INTO `preferences` (key,value) VALUES ('authorization_key','2c7b4db36907af8210e9b33291e258fe8807ea559bcb34a77e08a4456e1bb1b2');
        INSERT INTO `preferences` (key,value) VALUES ('tck','{\"tck_bytes\":[5,0,234,97,198,59,187,80,159,108,28,198,76,17,130,191,93,232,201,219,3,72,121,187,251,216,226,210,121,33,106,87,96,62,169,210,206,118,177,218,152,86,98,60,3,229,82,31,224,66,43,75,47,211,185,199,121,227,222,20,111,10,161,154,135,109]}');
        COMMIT;"; 

        let res = database.execute_batch(exported_db_sql);
        expect_log!(res, "Couldn't recreate db for version 0.3");

        let columns_2 = core_table_info("tcn", database.clone());
        assert_eq!(2, columns_2.len());
    }

    
    fn core_table_info(table_name: &str, database: Arc<Database>) -> Vec<String>{
        let columns = database.query("SELECT * FROM pragma_table_info(?)", params![table_name], |row: &Row|{to_table_information(row)}).unwrap();
        println!("Core rows: {:#?}", columns);
        columns
    }

    fn to_table_information(row: &Row) -> String {
        let ord: Result<i32, _> = row.get(0);
        let ord_value = expect_log!(ord, "Invalid row: no ordinal");

        let column_name_res = row.get(1);
        let column_name: String = expect_log!(column_name_res, "Invalid row: no column name");
        println!("Column {}: {}", ord_value, column_name);
        column_name
    }

}
