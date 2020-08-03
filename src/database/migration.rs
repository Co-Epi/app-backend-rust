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
                    self.drop_tcn_table();
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

    fn drop_tcn_table(&self){
        self.database
        .execute_sql(
            "drop table tcn;",
            params![],
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
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

}