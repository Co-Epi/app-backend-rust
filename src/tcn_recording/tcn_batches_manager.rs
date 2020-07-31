use super::observed_tcn_processor::ObservedTcn;
use crate::{
    database::tcn_dao::TcnDao,
    errors::ServicesError,
    expect_log,
    reports_update::exposure::{Exposure, ExposureGrouper},
};
use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TcnBatchesManager<T>
where
    T: TcnDao,
{
    tcn_dao: Arc<T>,
    tcns_batch: Mutex<HashMap<[u8; 16], ObservedTcn>>,
    exposure_grouper: ExposureGrouper,
}

impl<T> TcnBatchesManager<T>
where
    T: 'static + TcnDao,
{
    pub fn new(tcn_dao: Arc<T>, exposure_grouper: ExposureGrouper) -> TcnBatchesManager<T> {
        TcnBatchesManager {
            tcn_dao,
            tcns_batch: Mutex::new(HashMap::new()),
            exposure_grouper,
        }
    }

    pub fn flush(&self) -> Result<(), ServicesError> {
        let tcns = {
            let res = self.tcns_batch.lock();
            let mut tcns = expect_log!(res, "Couldn't lock tcns batch");
            let clone = tcns.clone();
            tcns.clear();
            clone
        };

        debug!("Flushing TCN batch into database: {:?}", tcns);

        // Do an in-memory merge with the DB TCNs and overwrite stored exposures with result.
        let merged = self.merge_with_db(tcns)?;
        self.tcn_dao.overwrite(merged)?;

        Ok(())
    }

    pub fn push(&self, tcn: ObservedTcn) {
        let res = self.tcns_batch.lock();
        let mut tcns = expect_log!(res, "Couldn't lock tcns batch");

        // TCNs in batch are merged to save memory and simplify processing / reduce logs.
        let merged_tcn = match tcns.get(&tcn.tcn.0) {
            Some(existing_tcn) => {
                match Self::merge_tcns(&self.exposure_grouper, existing_tcn.to_owned(), tcn.clone())
                {
                    Some(merged) => merged,
                    None => tcn,
                }
            }
            None => tcn,
        };
        tcns.insert(merged_tcn.tcn.0, merged_tcn);

        // debug!("Updated TCNs batch: {:?}", tcns);
    }

    // Used only in tests
    #[allow(dead_code)]
    pub fn len(&self) -> Result<usize, ServicesError> {
        match self.tcns_batch.lock() {
            Ok(tcns) => Ok(tcns.len()),
            Err(_) => Err(ServicesError::General(
                "Couldn't lock tcns batch".to_owned(),
            )),
        }
    }

    // Retrieves possible existing exposures from DB with same TCNs and does an in-memory merge.
    fn merge_with_db(
        &self,
        tcns: HashMap<[u8; 16], ObservedTcn>,
    ) -> Result<Vec<ObservedTcn>, ServicesError> {
        let tcns_vec: Vec<ObservedTcn> = tcns
            .iter()
            .map(|(_, observed_tcn)| observed_tcn.clone())
            .collect();

        let mut db_tcns = self
            .tcn_dao
            .find_tcns(tcns_vec.clone().into_iter().map(|tcn| tcn.tcn).collect())?;
        db_tcns.sort_by_key(|tcn| tcn.contact_start.value);

        let db_tcns_map: HashMap<[u8; 16], Vec<ObservedTcn>> = Self::to_hash_map(db_tcns);

        Ok(tcns_vec
            .into_iter()
            .map(|tcn|
            // Values in db_tcns_map can't be empty: we built the map based on existing TCNs
            Self::determine_tcns_to_write(self.exposure_grouper.clone(), db_tcns_map.clone(), tcn))
            .flatten()
            .collect())
    }

    // Expects:
    // - Values in db_tcns_map not empty
    // - db_tcns_map sorted by contact_start (ascending)
    fn determine_tcns_to_write(
        exposure_grouper: ExposureGrouper,
        db_tcns_map: HashMap<[u8; 16], Vec<ObservedTcn>>,
        tcn: ObservedTcn,
    ) -> Vec<ObservedTcn> {
        let db_tcns = db_tcns_map.get(&tcn.tcn.0);

        match db_tcns {
            // Matching exposures in DB
            Some(db_tcns) => {
                if let Some(last) = db_tcns.last() {
                    // If contiguous to last DB exposure, merge with it, otherwise append.
                    let tail =
                        match Self::merge_tcns(&exposure_grouper, last.to_owned(), tcn.clone()) {
                            Some(merged) => vec![merged],
                            None => vec![last.to_owned(), tcn],
                        };
                    let mut head: Vec<ObservedTcn> = db_tcns
                        .to_owned()
                        .into_iter()
                        .take(db_tcns.len() - 1)
                        .collect();
                    head.extend(tail);
                    head
                } else {
                    error!("Illegal state: value in db_tcns_map is empty");
                    panic!();
                }
            }
            // No matching exposures in DB: insert new TCN
            None => vec![tcn],
        }
    }

    fn to_hash_map(tcns: Vec<ObservedTcn>) -> HashMap<[u8; 16], Vec<ObservedTcn>> {
        let mut map: HashMap<[u8; 16], Vec<ObservedTcn>> = HashMap::new();
        for tcn in tcns {
            match map.get_mut(&tcn.tcn.0) {
                Some(tcns) => tcns.push(tcn),
                None => {
                    map.insert(tcn.tcn.0, vec![tcn]);
                }
            }
        }
        map
    }

    // Returns a merged TCN, if the TCNs are contiguous, None otherwise.
    // Assumes: tcn contact_start after db_tcn contact_start
    fn merge_tcns(
        exposure_grouper: &ExposureGrouper,
        db_tcn: ObservedTcn,
        tcn: ObservedTcn,
    ) -> Option<ObservedTcn> {
        if exposure_grouper.is_contiguous(&db_tcn, &tcn) {
            // Put db TCN and new TCN in an exposure as convenience to re-calculate measurements.
            let mut exposure = Exposure::create(db_tcn);
            exposure.push(tcn.clone());
            let measurements = exposure.measurements();
            Some(ObservedTcn {
                tcn: tcn.tcn,
                contact_start: measurements.contact_start,
                contact_end: measurements.contact_end,
                min_distance: measurements.min_distance,
                avg_distance: measurements.avg_distance,
                total_count: measurements.total_count,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::{database::Database, tcn_dao::TcnDaoImpl},
        reports_interval::UnixTime,
    };
    use rusqlite::Connection;
    use tcn::TemporaryContactNumber;

    #[test]
    fn test_push_merges_existing_tcn_in_batch_manager() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database);

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

        let tcns = batches_manager.tcns_batch.lock().unwrap();
        assert_eq!(
            tcns[&[0; 16]],
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 5000 },
                min_distance: 2.0,
                avg_distance: 0.57875, // (0.1 + 0.62 + 0.8 + 0.21 + 0.8 + 1.2 + 0.5 + 0.4) / (5 + 3)
                total_count: 8         // 5 + 3
            }
        );
    }

    #[test]
    fn test_flush_clears_tcns() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database);

        let batches_manager =
            TcnBatchesManager::new(Arc::new(tcn_dao), ExposureGrouper { threshold: 1000 });

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 2.3,
            total_count: 1,
        });
        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let len_res = batches_manager.len();
        assert!(len_res.is_ok());
        assert_eq!(0, len_res.unwrap())
    }

    #[test]
    fn test_flush_adds_entries_to_db() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database));

        let batches_manager =
            TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper { threshold: 1000 });

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 2.3,
            total_count: 1,
        };
        batches_manager.push(tcn.clone());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let stored_tcns_res = tcn_dao.all();
        assert!(stored_tcns_res.is_ok());

        let stored_tcns = stored_tcns_res.unwrap();
        assert_eq!(1, stored_tcns.len());
        assert_eq!(tcn, stored_tcns[0]);
    }

    #[test]
    fn test_flush_updates_correctly_existing_entry() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database));

        let batches_manager =
            TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper { threshold: 1000 });

        let stored_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 1.25, // (2.3 + 0.7 + 1 + 1) / 4
            total_count: 4,
        };
        let save_res = tcn_dao.overwrite(vec![stored_tcn]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
            avg_distance: 1.0, // (1.12 + 0.88 + 1) / 3
            total_count: 3,
        };
        batches_manager.push(tcn.clone());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(1, loaded_tcns.len());

        assert_eq!(
            loaded_tcns[0],
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 5000 },
                min_distance: 1.12,
                avg_distance: 1.14285714, // (2.3 + 0.7 + 1 + 1 + 1.12 + 0.88 + 1) / (4 + 3)
                total_count: 7,
            }
        );
    }

    #[test]
    fn test_flush_does_not_affect_different_stored_tcn() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database));

        let batches_manager =
            TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper { threshold: 1000 });

        let stored_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([1; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 2.3,
            total_count: 1,
        };
        let save_res = tcn_dao.overwrite(vec![stored_tcn]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
            avg_distance: 1.12,
            total_count: 1,
        };
        batches_manager.push(tcn.clone());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(2, loaded_tcns.len());

        assert_eq!(
            loaded_tcns[0],
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 2600 },
                min_distance: 2.3,
                avg_distance: 2.3,
                total_count: 1,
            }
        );
        assert_eq!(
            loaded_tcns[1],
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 3000 },
                contact_end: UnixTime { value: 5000 },
                min_distance: 1.12,
                avg_distance: 1.12,
                total_count: 1
            }
        );
    }

    #[test]
    fn test_flush_updates_correctly_2_stored_1_updated() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database));

        let batches_manager =
            TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper { threshold: 1000 });

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
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
            avg_distance: 2.3,
            total_count: 1,
        };
        let save_res = tcn_dao.overwrite(vec![stored_tcn1.clone(), stored_tcn2.clone()]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 7000 },
            min_distance: 1.12,
            avg_distance: 1.12,
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
                contact_end: UnixTime { value: 7000 },
                min_distance: 0.4,
                avg_distance: 0.76, // (0.4, + 1.12) / (1 + 1)
                total_count: 2      // 1 + 1
            }
        );
        assert_eq!(loaded_tcns[1], stored_tcn2);
    }
}
