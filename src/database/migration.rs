use super::database::Database;
use crate::expect_log;
use log::*;
use rusqlite::params;
use std::sync::Arc;

pub struct Migration {
    database: Arc<Database>,
}

impl Migration {
    pub fn new(database: Arc<Database>) -> Migration {
        Migration { database: database }
    }

    pub fn run_db_migrations(&self, required_db_version: i32) {
        let pragma_variable_name = "user_version";
        let db_version_before_migration = self.database.core_pragma_query(pragma_variable_name);

        let db_version_after_migration =
            self.migrate_db(db_version_before_migration, required_db_version);

        if db_version_after_migration > db_version_before_migration {
            self.database
                .core_pragma_update(pragma_variable_name, &db_version_after_migration);
        }
    }

    fn migrate_db(&self, from_version: i32, to_version: i32) -> i32 {
        if from_version >= to_version {
            warn!(
                "DB version is greater than required: {} >= {}",
                from_version, to_version
            );
            return from_version;
        }
        let mut db_version = from_version;
        while db_version < to_version {
            debug!("DB version is {}", db_version);
            match db_version {
                0 => {
                    self.migration_0_drop_tcn_table();
                    db_version += 1;
                }
                _ => {
                    warn!("Migration from DB version {} not handled!", db_version);
                    break;
                }
            }
        }

        db_version
    }

    fn migration_0_drop_tcn_table(&self) {
        let exec_res = self
            .database
            .execute_sql("drop table if exists tcn;", params![]);
        expect_log!(exec_res, "Dropping tcn table failed!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tcn_dao::TcnDaoImpl;
    use crate::reports_interval::UnixTime;
    use crate::reports_update::exposure::ExposureGrouper;
    use crate::simple_logger;
    use crate::tcn_recording::observed_tcn_processor::ObservedTcn;
    use crate::tcn_recording::tcn_batches_manager::TcnBatchesManager;
    use rusqlite::{params, Connection, Row};
    use tcn::TemporaryContactNumber;

    #[test]
    fn test_migration_to_the_same_or_lower_version() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let pragma_variable_name = "user_version";
        let db_version: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(0, db_version);
        let target_db_version = 17;
        database.core_pragma_update(pragma_variable_name, &target_db_version);
        let db_version_before_migration: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(target_db_version, db_version_before_migration);

        let migration_handler = Migration::new(database.clone());
        //migrate to same version
        migration_handler.run_db_migrations(target_db_version);

        let db_version_after_migration: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(target_db_version, db_version_after_migration);

        //migrate to lower version
        migration_handler.run_db_migrations(target_db_version - 1);
        let db_version_after_migration: i32 = database.core_pragma_query(pragma_variable_name);
        assert_eq!(target_db_version, db_version_after_migration);
    }

    #[test]
    fn test_tcn_flush_after_migration() {
        simple_logger::setup();
        let table_name = "tcn";
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        //set up database structure for version 0.3
        prep_data_structure_for_app_version_03(database.clone());

        //migrate DB
        let migration_handler = Migration::new(database.clone());
        migration_handler.run_db_migrations(3);
        core_table_info(table_name, database.clone());

        //verify flushing works
        let tcn_dao = TcnDaoImpl::new(database.clone());
        core_table_info(table_name, database);
        let batches_manager =
            TcnBatchesManager::new(Arc::new(tcn_dao), ExposureGrouper { threshold: 1000 });

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 0.506, // (0.1 + 0.62 + 0.8 + 0.21 + 0.8) / 5
            total_count: 5,
        });

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 2.0,
            avg_distance: 0.7, // (1.2 + 0.5 + 0.4) / 3
            total_count: 3,
        });

        let len_res = batches_manager.len();
        assert!(len_res.is_ok());
        assert_eq!(1, len_res.unwrap());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());
        expect_log!(flush_res, "Tcn flushing failed");
    }

    #[test]
    fn test_migration_with_tcn_table_altering() {
        simple_logger::setup();
        let table_name = "tcn";
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        //set up database structure for version 0.3
        prep_data_structure_for_app_version_03(database.clone());
        let table_columns_ver_03 = core_table_info(table_name, database.clone());
        assert_eq!(2, table_columns_ver_03.len());

        //alter tcn table
        migration_0_alter_tcn_table(database.clone());
        let table_columns_after_migration = core_table_info(table_name, database);
        assert_eq!(6, table_columns_after_migration.len());
    }

    fn migration_0_alter_tcn_table(database: Arc<Database>) {
        let exec_res = database.execute_sql(
            "alter table tcn rename column contact_time to contact_start;",
            params![],
        );
        expect_log!(exec_res, "Renaming tcn table contact_time failed!");
        let exec_res = database.execute_sql(
            "alter table tcn add column contact_end integer not null default 0;",
            params![],
        );
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = database.execute_sql(
            "alter table tcn add column min_distance real default 32.0;",
            params![],
        );
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = database.execute_sql(
            "alter table tcn add column avg_distance real default 56.0;",
            params![],
        );
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = database.execute_sql(
            "alter table tcn add column total_count integer default 48;",
            params![],
        );
        expect_log!(exec_res, "Altering tcn table failed");
    }

    fn prep_data_structure_for_app_version_03(database: Arc<Database>) {
        let exported_db_sql = "BEGIN TRANSACTION;
        CREATE TABLE IF NOT EXISTS tcn(
                        tcn text not null,
                        contact_time integer not null
                    );
        CREATE TABLE IF NOT EXISTS preferences(
                        key text primary key,
                        value text not null
                    );
        COMMIT;";

        let res = database.execute_batch(exported_db_sql);
        expect_log!(res, "Couldn't recreate db for version 0.3");
    }

    fn core_table_info(table_name: &str, database: Arc<Database>) -> Vec<String> {
        let columns = database
            .query(
                "SELECT * FROM pragma_table_info(?)",
                params![table_name],
                |row: &Row| to_table_information(row),
            )
            .unwrap();
        debug!("{} table columns: {:#?}", table_name, columns);
        columns
    }

    fn to_table_information(row: &Row) -> String {
        let ord: Result<i32, _> = row.get(0);
        let ord_value = expect_log!(ord, "Invalid row: no ordinal");

        let column_name_res = row.get(1);
        let column_name: String = expect_log!(column_name_res, "Invalid row: no column name");
        debug!("Column {}: {}", ord_value, column_name);
        column_name
    }
}
