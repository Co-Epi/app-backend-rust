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
        let mut db_version = self.database.core_pragma_query(pragma_variable_name);
        while db_version < required_db_version {
            debug!("DB version is {}", db_version);
            match db_version {
                0 => {
                    self.migrate_data_03_to_04();
                    db_version += 1;
                }
                _ => {
                    warn!("Migration from DB version {} not handled!", db_version);
                    break;
                }
            }
        }

        self.database
            .core_pragma_update(pragma_variable_name, &db_version);
    }

    fn migrate_data_03_to_04(&self) {
        self.database
            .execute_sql(
                "alter table tcn rename column contact_time to contact_start;",
                params![],
            )
            .unwrap();
        self.database
            .execute_sql(
                "alter table tcn add column contact_end integer not null default 0;",
                params![],
            )
            .unwrap();
        self.database
            .execute_sql(
                "alter table tcn add column min_distance real default 32.0;",
                params![],
            )
            .unwrap();
        self.database
            .execute_sql(
                "alter table tcn add column avg_distance real default 56.0;",
                params![],
            )
            .unwrap();
        self.database
            .execute_sql(
                "alter table tcn add column total_count integer default 48;",
                params![],
            )
            .unwrap();
    }
}
