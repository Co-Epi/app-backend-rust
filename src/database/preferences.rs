use super::database::Database;
use crate::{byte_vec_to_32_byte_array, expect_log, reports_interval::ReportsInterval};
use log::*;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{option::Option, sync::Arc};

pub const TCK_SIZE_IN_BYTES: usize = 66;

big_array! { BigArray; TCK_SIZE_IN_BYTES}
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MyConfig {
    last_completed_reports_interval: Option<ReportsInterval>,
    autorization_key: Option<[u8; 32]>,
    tck: Option<TckBytesWrapper>,
}

//Wrapper struct added to enable custom serialization of a large byte array
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TckBytesWrapper {
    #[serde(with = "BigArray")]
    pub tck_bytes: [u8; TCK_SIZE_IN_BYTES],
}

impl fmt::Debug for TckBytesWrapper {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.tck_bytes[..].fmt(formatter)
    }
}

impl AsRef<[u8]> for TckBytesWrapper {
    fn as_ref(&self) -> &[u8] {
        &self.tck_bytes
    }
}

impl PartialEq for TckBytesWrapper {
    fn eq(&self, other: &Self) -> bool {
        &self.tck_bytes[..] == &other.tck_bytes[..]
    }
}
impl Eq for TckBytesWrapper {}

impl Default for MyConfig {
    fn default() -> Self {
        Self {
            last_completed_reports_interval: None,
            autorization_key: None,
            tck: None,
        }
    }
}

pub struct PreferencesDao {
    db: Arc<Database>,
}

impl PreferencesDao {
    pub fn load(&self, key: &str) -> Option<String> {
        let result = self.db.query_row(
            "select value from preferences where key=?1",
            &[key],
            |row| {
                let res = row.get(0);
                let value: String = expect_log!(res, "Invalid row");
                Ok(value)
            },
        );

        if let Err(e) = &result {
            match e {
                // Empty result: not an error
                rusqlite::Error::QueryReturnedNoRows => {}
                _ => error!("Error loading preference: {:?}", e),
            }
        }

        result.ok()
    }

    pub fn save(&self, key: &str, value: &str) {
        let res = self.db.execute_sql(
            "insert or replace into preferences(key, value) values(?1, ?2)",
            params![key, value],
        );
        expect_log!(res, "Couldn't insert preference");
    }

    pub fn new(db: Arc<Database>) -> PreferencesDao {
        Self::create_table_if_not_exists(&db);
        PreferencesDao { db }
    }

    fn create_table_if_not_exists(db: &Arc<Database>) {
        let res = db.execute_sql(
            "create table if not exists preferences(
                key text primary key,
                value text not null
            )",
            params![],
        );
        expect_log!(res, "Couldn't create preferences table");
    }
}

pub trait Preferences {
    fn last_completed_reports_interval(&self) -> Option<ReportsInterval>;
    fn set_last_completed_reports_interval(&self, value: ReportsInterval);

    // TODO encrypted
    fn authorization_key(&self) -> Option<[u8; 32]>;
    fn set_autorization_key(&self, value: [u8; 32]);

    fn tck(&self) -> Option<TckBytesWrapper>;
    fn set_tck(&self, value: TckBytesWrapper);
}

pub struct PreferencesImpl {
    pub dao: PreferencesDao,
}

impl Preferences for PreferencesImpl {
    fn last_completed_reports_interval(&self) -> Option<ReportsInterval> {
        let str = self.dao.load("last_completed_reports_interval");
        str.map(|str| {
            let res = serde_json::from_str(str.as_ref());
            expect_log!(res, "Invalid interval str")
        })
    }

    fn set_last_completed_reports_interval(&self, value: ReportsInterval) {
        let res = serde_json::to_string(&value);
        let str = expect_log!(res, "Couldn't serialize interval");
        self.dao
            .save("last_completed_reports_interval", str.as_ref())
    }

    fn authorization_key(&self) -> Option<[u8; 32]> {
        let str = self.dao.load("authorization_key");
        let bytes = str.map(|str| {
            let res = hex::decode(str);
            expect_log!(res, "Invalid interval str")
        });
        bytes.map(|bytes| byte_vec_to_32_byte_array(bytes))
    }

    fn set_autorization_key(&self, value: [u8; 32]) {
        self.dao
            .save("authorization_key", hex::encode(&value).as_ref())
    }

    fn tck(&self) -> Option<TckBytesWrapper> {
        let str = self.dao.load("tck");
        str.map(|str| {
            let res = serde_json::from_str(str.as_ref());
            expect_log!(res, "Invalid tck wrapper str")
        })
    }

    fn set_tck(&self, value: TckBytesWrapper) {
        let res = serde_json::to_string(&value);
        let str = expect_log!(res, "Couldn't serialize tck wrapper");
        self.dao.save("tck", str.as_ref())
    }
}

pub struct PreferencesTckMock {
    pub tck_bytes: TckBytesWrapper,
}

impl Preferences for PreferencesTckMock {
    fn last_completed_reports_interval(&self) -> Option<ReportsInterval> {
        let reports_interval = ReportsInterval {
            number: 8899222,
            length: 12232,
        };
        Option::Some(reports_interval)
    }

    fn set_last_completed_reports_interval(&self, _: ReportsInterval) {
        return;
    }

    fn authorization_key(&self) -> std::option::Option<[u8; 32]> {
        let bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];
        return Option::Some(bytes);
    }

    fn set_autorization_key(&self, _value: [u8; 32]) {
        return;
    }

    fn tck(&self) -> std::option::Option<TckBytesWrapper> {
        Some(self.tck_bytes)
    }

    fn set_tck(&self, _value: TckBytesWrapper) {
        return;
    }
}

#[derive(Clone)]
pub struct PreferencesNoopMock {}
impl Preferences for PreferencesNoopMock {
    fn last_completed_reports_interval(&self) -> Option<ReportsInterval> {
        Option::None
    }

    fn set_last_completed_reports_interval(&self, _: ReportsInterval) {}

    fn authorization_key(&self) -> std::option::Option<[u8; 32]> {
        Option::None
    }

    fn set_autorization_key(&self, _value: [u8; 32]) {}

    fn tck(&self) -> std::option::Option<TckBytesWrapper> {
        Option::None
    }

    fn set_tck(&self, _value: TckBytesWrapper) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tcn_ext::tcn_keys::TckBytesWrapperExt;
    use rusqlite::Connection;

    #[test]
    fn test_saves_last_completed_reports_interval() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let preferences_dao = PreferencesDao::new(database.clone());
        let preferences = PreferencesImpl {
            dao: preferences_dao,
        };

        let interval = ReportsInterval {
            number: 1,
            length: 10,
        };
        preferences.set_last_completed_reports_interval(interval);

        assert_eq!(
            preferences.last_completed_reports_interval().unwrap(),
            interval
        );
    }

    #[test]
    fn test_saves_tck() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let preferences_dao = PreferencesDao::new(database.clone());
        let preferences = PreferencesImpl {
            dao: preferences_dao,
        };

        let tck_bytes_wrapper = create_test_tck();

        preferences.set_tck(tck_bytes_wrapper);

        assert_eq!(preferences.tck().unwrap(), tck_bytes_wrapper);
    }

    #[test]
    fn test_saves_autorization_key() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let preferences_dao = PreferencesDao::new(database.clone());
        let preferences = PreferencesImpl {
            dao: preferences_dao,
        };

        let rak_bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];

        preferences.set_autorization_key(rak_bytes);

        assert_eq!(preferences.authorization_key().unwrap(), rak_bytes);
    }

    fn create_test_tck() -> TckBytesWrapper {
        let rak_bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];

        let tck_inner_bytes = [
            34, 166, 47, 23, 224, 52, 240, 95, 140, 186, 95, 243, 26, 13, 174, 128, 224, 229, 158,
            248, 117, 7, 118, 110, 108, 57, 67, 206, 129, 22, 84, 13,
        ];

        let version_bytes: [u8; 2] = [1, 0];

        let version_vec = version_bytes.to_vec();
        let rak_vec = rak_bytes.to_vec();
        let tck_inner_vec = tck_inner_bytes.to_vec();

        let complete_tck_vec = [&version_vec[..], &rak_vec[..], &tck_inner_vec[..]].concat();

        TckBytesWrapper::with_bytes(complete_tck_vec)
    }
}
