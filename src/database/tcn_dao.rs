use crate::{
    byte_vec_to_16_byte_array,
    errors::{ServicesError},
    expect_log,
    reports_interval, tcn_recording::observed_tcn_processor::ObservedTcn,
};
use log::*;
use reports_interval::UnixTime;
use rusqlite::{params, Row, NO_PARAMS, types::Value};
use std::{
    sync::Arc,
    rc::Rc,
};
use tcn::TemporaryContactNumber;
use super::database::Database;

pub trait TcnDao: Send + Sync {
    fn all(&self) -> Result<Vec<ObservedTcn>, ServicesError>;
    fn find_tcns(
        &self,
        with: Vec<TemporaryContactNumber>,
    ) -> Result<Vec<ObservedTcn>, ServicesError>;
    // Removes all matching TCNs (same TCN bytes) and stores observed_tcns 
    fn overwrite(&self, observed_tcns: Vec<ObservedTcn>) -> Result<(), ServicesError>;
}

pub struct TcnDaoImpl {
    db: Arc<Database>,
}

impl TcnDaoImpl {
    fn create_table_if_not_exists(db: &Arc<Database>) {
        // TODO use blob for tcn? https://docs.rs/rusqlite/0.23.1/rusqlite/blob/index.html
        // TODO ideally FFI should send byte arrays too
        let res = db.execute_sql(
            "create table if not exists tcn(
                tcn text not null,
                contact_start integer not null,
                contact_end integer not null,
                min_distance real not null,
                avg_distance real not null,
                total_count integer not null
            )",
            params![],
        );
        expect_log!(res, "Couldn't create tcn table");
    }

    fn to_tcn(row: &Row) -> ObservedTcn {
        let tcn: Result<String, _> = row.get(0);
        let tcn_value = expect_log!(tcn, "Invalid row: no TCN");
        let tcn = Self::db_tcn_str_to_tcn(tcn_value);

        let contact_start_res = row.get(1);
        let contact_start: i64 = expect_log!(contact_start_res, "Invalid row: no contact start");

        let contact_end_res = row.get(2);
        let contact_end: i64 = expect_log!(contact_end_res, "Invalid row: no contact end");

        let min_distance_res = row.get(3);
        let min_distance: f64 = expect_log!(min_distance_res, "Invalid row: no min distance");

        let avg_distance_res = row.get(4);
        let avg_distance: f64 = expect_log!(avg_distance_res, "Invalid row: no avg distance");

        let total_count_res = row.get(5);
        let total_count: i64 = expect_log!(total_count_res, "Invalid row: no total count");

        ObservedTcn {
            tcn,
            contact_start: UnixTime {
                value: contact_start as u64,
            },
            contact_end: UnixTime {
                value: contact_end as u64,
            },
            min_distance: min_distance as f32,
            avg_distance: avg_distance as f32,
            total_count: total_count as usize,
        }
    }

    // TCN string loaded from DB is assumed to be valid
    fn db_tcn_str_to_tcn(str: String) -> TemporaryContactNumber {
        let tcn_value_bytes_vec_res = hex::decode(str);
        let tcn_value_bytes_vec = expect_log!(tcn_value_bytes_vec_res, "Invalid stored TCN format");
        let tcn_value_bytes = byte_vec_to_16_byte_array(tcn_value_bytes_vec);
        TemporaryContactNumber(tcn_value_bytes)
    }

    pub fn new(db: Arc<Database>) -> TcnDaoImpl {
        Self::create_table_if_not_exists(&db);
        TcnDaoImpl { db }
    }
}

impl TcnDao for TcnDaoImpl {
    fn all(&self) -> Result<Vec<ObservedTcn>, ServicesError> {
        self.db
            .query(
                "select tcn, contact_start, contact_end, min_distance, avg_distance, total_count from tcn",
                NO_PARAMS,
                |row| Self::to_tcn(row),
            )
            .map_err(ServicesError::from)
    }

    fn find_tcns(
        &self,
        with: Vec<TemporaryContactNumber>,
    ) -> Result<Vec<ObservedTcn>, ServicesError> {
        let tcn_strs: Vec<Value> = with.into_iter().map(|tcn| 
            Value::Text(hex::encode(tcn.0))
        )
        .collect();

        self.db
            .query(
                "select tcn, contact_start, contact_end, min_distance, avg_distance, total_count from tcn where tcn in rarray(?);",
                params![Rc::new(tcn_strs)],
                |row| Self::to_tcn(row),
            )
            .map_err(ServicesError::from)
    }

    fn overwrite(&self, observed_tcns: Vec<ObservedTcn>) -> Result<(), ServicesError> {
        debug!("Overwriting db exposures with same TCNs, with: {:?}", observed_tcns);

        let tcn_strs: Vec<Value> = observed_tcns.clone().into_iter().map(|tcn| 
            Value::Text(hex::encode(tcn.tcn.0))
        )
        .collect();

        self.db.transaction(|t| {
            // Delete all the exposures for TCNs
            let delete_res = t.execute("delete from tcn where tcn in rarray(?);", params![Rc::new(tcn_strs)]);
            if delete_res.is_err() {
                return Err(ServicesError::General("Delete TCNs failed".to_owned()))
            } 

            // Insert up to date exposures
            for tcn in observed_tcns {
                let tcn_str = hex::encode(tcn.tcn.0);
                let insert_res = t.execute("insert into tcn(tcn, contact_start, contact_end, min_distance, avg_distance, total_count) values(?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    tcn_str,
                    tcn.contact_start.value as i64,
                    tcn.contact_end.value as i64,
                    tcn.min_distance as f64, // db requires f64 / real
                    tcn.avg_distance as f64, // db requires f64 / real
                    tcn.total_count as i64
                ]);

                if insert_res.is_err() {
                    return Err(ServicesError::General("Insert TCN failed".to_owned()))
                }
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::{tcn_recording::tcn_batches_manager::TcnBatchesManager, reports_update::exposure::ExposureGrouper};

    #[test]
    fn saves_and_loads_observed_tcn() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database.clone());

        let observed_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([
                24, 229, 125, 245, 98, 86, 219, 221, 172, 25, 232, 150, 206, 66, 164, 173,
            ]),
            contact_start: UnixTime { value: 1590528300 },
            contact_end: UnixTime { value: 1590528301 },
            min_distance: 0.0,
            avg_distance: 0.0,
            total_count: 1,
        };

        let save_res = tcn_dao.overwrite(vec![observed_tcn.clone()]);
        assert!(save_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();

        assert_eq!(loaded_tcns.len(), 1);
        assert_eq!(loaded_tcns[0], observed_tcn);
    }

    #[test]
    fn saves_and_loads_multiple_tcns() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database.clone());

        let observed_tcn_1 = ObservedTcn {
            tcn: TemporaryContactNumber([
                24, 229, 125, 245, 98, 86, 219, 221, 172, 25, 232, 150, 206, 66, 164, 173,
            ]),
            contact_start: UnixTime { value: 1590528300 },
            contact_end: UnixTime { value: 1590528301 },
            min_distance: 0.0,
            avg_distance: 0.0,
            total_count: 1,
        };
        let observed_tcn_2 = ObservedTcn {
            tcn: TemporaryContactNumber([
                43, 229, 125, 245, 98, 86, 100, 1, 172, 25, 0, 150, 123, 66, 34, 12,
            ]),
            contact_start: UnixTime { value: 1590518190 },
            contact_end: UnixTime { value: 1590518191 },
            min_distance: 0.0,
            avg_distance: 0.0,
            total_count: 1,
        };
        let observed_tcn_3 = ObservedTcn {
            tcn: TemporaryContactNumber([
                11, 246, 125, 123, 102, 86, 100, 1, 34, 25, 21, 150, 99, 66, 34, 0,
            ]),
            contact_start: UnixTime { value: 2230522104 },
            contact_end: UnixTime { value: 2230522105 },
            min_distance: 0.0,
            avg_distance: 0.0,
            total_count: 1,
        };

        let save_res_1 = tcn_dao.overwrite(vec![observed_tcn_1.clone()]);
        let save_res_2 = tcn_dao.overwrite(vec![observed_tcn_2.clone()]);
        let save_res_3 = tcn_dao.overwrite(vec![observed_tcn_3.clone()]);
        assert!(save_res_1.is_ok());
        assert!(save_res_2.is_ok());
        assert!(save_res_3.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();

        assert_eq!(loaded_tcns.len(), 3);
        assert_eq!(loaded_tcns[0], observed_tcn_1);
        assert_eq!(loaded_tcns[1], observed_tcn_2);
        assert_eq!(loaded_tcns[2], observed_tcn_3);
    }

    #[test]
    fn test_finds_tcn() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let stored_tcn1 = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1000 },
            contact_end: UnixTime { value: 6000 },
            min_distance: 0.4,
            avg_distance: 0.4,
            total_count: 1,
        };

        let stored_tcn2 = ObservedTcn {
            tcn: TemporaryContactNumber([1; 16]),
            contact_start: UnixTime { value: 2000 },
            contact_end: UnixTime { value: 3000 },
            min_distance: 1.8,
            avg_distance: 1.8,
            total_count: 1,
        };

        let stored_tcn3 = ObservedTcn {
            tcn: TemporaryContactNumber([2; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 2.3,
            total_count: 1,
        };

        let save_res = tcn_dao.overwrite(vec![
            stored_tcn1.clone(),
            stored_tcn2.clone(),
            stored_tcn3.clone(),
        ]);
        assert!(save_res.is_ok());

        let res = tcn_dao.find_tcns(vec![
            TemporaryContactNumber([0; 16]),
            TemporaryContactNumber([2; 16]),
        ]);
        assert!(res.is_ok());

        let mut tcns = res.unwrap();

        // Sqlite doesn't guarantee insertion order, so sort
        // start value not meaningul here, other than for reproducible sorting
        tcns.sort_by_key(|tcn| tcn.contact_start.value);

        assert_eq!(2, tcns.len());
        assert_eq!(stored_tcn1, tcns[0]);
        assert_eq!(stored_tcn3, tcns[1]);
    }
    
    #[test]
    fn test_multiple_exposures_updated_correctly() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let batches_manager =
            TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper { threshold: 1000 });

        let stored_tcn1 = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1000 },
            contact_end: UnixTime { value: 3000 },
            min_distance: 0.4,
            avg_distance: 0.4,
            total_count: 1,
        };

        let stored_tcn2 = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 5000 },
            contact_end: UnixTime { value: 7000 },
            min_distance: 2.0,
            avg_distance: 2.0,
            total_count: 1,
        };
        let save_res = tcn_dao.overwrite(vec![stored_tcn1.clone(), stored_tcn2.clone()]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 7500 },
            contact_end: UnixTime { value: 9000 },
            min_distance: 1.0,
            avg_distance: 1.0,
            total_count: 1,
        };

        batches_manager.push(tcn.clone());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let mut loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(2, loaded_tcns.len());

        // Sqlite doesn't guarantee insertion order, so sort
        // start value not meaningul here, other than for reproducible sorting
        loaded_tcns.sort_by_key(|tcn| tcn.contact_start.value);

        assert_eq!(
            loaded_tcns[0],
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 3000 },
                min_distance: 0.4,
                avg_distance: 0.4,
                total_count: 1
            }
        );
        // The new TCN was merged with stored_tcn2
        assert_eq!(
            loaded_tcns[1],
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 5000 },
                contact_end: UnixTime { value: 9000 },
                min_distance: 1.0,
                avg_distance: 1.5, // (2.0 + 1.0) / (1 + 1),
                total_count: 2     // 1 + 1
            }
        );
    }
}
