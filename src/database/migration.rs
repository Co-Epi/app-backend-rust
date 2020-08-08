use crate::expect_log;
use super::database::Database;
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
   
        let db_version_after_migration = self.migrate_db(db_version_before_migration, required_db_version);

        if db_version_after_migration > db_version_before_migration {
            self.database
            .core_pragma_update(pragma_variable_name, &db_version_after_migration);
        }

    }

    fn migrate_db(&self, from_version:i32, to_version: i32) -> i32{
        if from_version >= to_version {
            warn!("DB version is greater than required: {} >= {}", from_version, to_version);
            return from_version
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

    fn migration_0_drop_tcn_table(&self){
        let exec_res = self.database
        .execute_sql(
            "drop table if exists tcn;",
            params![],
        );
        expect_log!(exec_res, "Dropping tcn table failed!");
    }

    fn migration_0_alter_tcn_table(&self){
        let exec_res = self.
        database.execute_sql("alter table tcn rename column contact_time to contact_start;", params![]);
        expect_log!(exec_res, "Renaming tcn table contact_time failed!");
        let exec_res = self.database.execute_sql("alter table tcn add column contact_end integer not null default 0;", params![]);
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = self.database.execute_sql("alter table tcn add column min_distance real default 32.0;", params![]);
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = self.database.execute_sql("alter table tcn add column avg_distance real default 56.0;", params![]);
        expect_log!(exec_res, "Altering tcn table failed");
        let exec_res = self.database.execute_sql("alter table tcn add column total_count integer default 48;", params![]);
        expect_log!(exec_res, "Altering tcn table failed");
        // let columns_6 = core_table_info("tcn", self.database.clone());
        // assert_eq!(6, columns_6.len());

        // let _migrated_tcns = database.query("SELECT * FROM tcn",
        // NO_PARAMS,
        // |row| to_tcn_conditional(row));

        // println!("migrated_tcns: {:#?}", migrated_tcns);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{Connection, params, Row};
    use crate::database::tcn_dao::TcnDaoImpl;
    use crate::reports_interval::UnixTime;
    use crate::tcn_recording::observed_tcn_processor::ObservedTcn;
    use crate::tcn_recording::tcn_batches_manager::TcnBatchesManager;
    use crate::reports_update::exposure::ExposureGrouper;
    use tcn::TemporaryContactNumber;
    
    #[test]
    fn test_migration_to_the_same_or_lower_version(){
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
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        //set up database structure for version 0.3
        prep_data_structure_for_app_version_03(database.clone());

        //migrate DB
        let migration_handler = Migration::new(database.clone());
        migration_handler.run_db_migrations(3);
        core_table_info("tcn", database.clone());

        //verify flushing works
        let tcn_dao = TcnDaoImpl::new(database.clone());
        core_table_info("tcn", database.clone());
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

   
    fn prep_data_structure_for_app_version_03(database: Arc<Database>){

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

        let columns_2 = core_table_info("tcn", database);
        assert_eq!(2, columns_2.len());
    }

    
    fn core_table_info(table_name: &str, database: Arc<Database>) -> Vec<String>{
        let columns = database.query("SELECT * FROM pragma_table_info(?)", params![table_name], |row: &Row|{to_table_information(row)}).unwrap();
        println!("{} table columns: {:#?}", table_name, columns);
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