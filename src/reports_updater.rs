use crate::{
    byte_vec_to_16_byte_array,
    errors::{Error, ServicesError},
    expect_log,
    networking::{NetworkingError, TcnApi},
    preferences::{Database, Preferences},
    reporting::{
        memo::{Memo, MemoMapper},
        public_report::PublicReport,
    },
    reports_interval,
};
use exposure::Exposure;
use log::*;
use rayon::prelude::*;
use reports_interval::{ReportsInterval, UnixTime};
use rusqlite::{params, Row, NO_PARAMS};
use serde::Serialize;
use std::collections::HashMap;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};
use tcn::{SignedReport, TemporaryContactNumber};
use timer::Timer;

pub trait TcnMatcher {
    fn match_reports(
        &self,
        tcns: Vec<ObservedTcn>,
        reports: Vec<SignedReport>,
    ) -> Result<Vec<MatchedReport>, ServicesError>;
}

#[derive(Debug, Clone)]
pub struct MatchedReport {
    report: SignedReport,
    tcns: Vec<ObservedTcn>,
}

pub struct TcnMatcherRayon {}

impl TcnMatcher for TcnMatcherRayon {
    fn match_reports(
        &self,
        tcns: Vec<ObservedTcn>,
        reports: Vec<SignedReport>,
    ) -> Result<Vec<MatchedReport>, ServicesError> {
        Self::match_reports_with(tcns, reports)
    }
}

impl TcnMatcherRayon {
    pub fn match_reports_with(
        tcns: Vec<ObservedTcn>,
        reports: Vec<SignedReport>,
    ) -> Result<Vec<MatchedReport>, ServicesError> {
        let observed_tcns_map: HashMap<[u8; 16], ObservedTcn> =
            tcns.into_iter().map(|e| (e.tcn.0, e)).collect();

        let observed_tcns_map = Arc::new(observed_tcns_map);

        let res: Vec<Option<MatchedReport>> = reports
            .par_iter()
            .map(|report| Self::match_report_with(&observed_tcns_map, report))
            .collect();

        let res: Vec<MatchedReport> = res
            .into_iter()
            .filter_map(|option| option) // drop None (reports that didn't match)
            .collect();

        Ok(res)
    }

    pub fn match_report_with(
        observed_tcns_map: &HashMap<[u8; 16], ObservedTcn>,
        report: &SignedReport,
    ) -> Option<MatchedReport> {
        let rep = report.clone().verify();
        match rep {
            Ok(rep) => {
                let mut tcns: Vec<ObservedTcn> = vec![];
                for tcn in rep.temporary_contact_numbers() {
                    if let Some(observed_tcn) = observed_tcns_map.get(&tcn.0) {
                        tcns.push(observed_tcn.to_owned());
                    }
                }
                if tcns.is_empty() {
                    None
                } else {
                    Some(MatchedReport {
                        report: report.clone(),
                        tcns,
                    })
                }
            }
            Err(error) => {
                error!("Report can't be matched. Verification failed: {:?}", error);
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObservedTcn {
    tcn: TemporaryContactNumber,
    contact_start: UnixTime,
    contact_end: UnixTime,
    min_distance: f32,
}

pub trait ObservedTcnProcessor {
    fn save(&self, tcn_str: &str, distance: f32) -> Result<(), ServicesError>;
}

#[derive(Copy, Clone)]
struct Element {}

pub struct TcnBatchesManager<T>
where
    T: TcnDao,
{
    tcn_dao: Arc<T>,
    tcns_batch: Arc<Mutex<Vec<ObservedTcn>>>, // TODO Arc needed?
    exposure_grouper: ExposureGrouper,
}

impl<T> TcnBatchesManager<T>
where
    T: 'static + TcnDao,
{
    pub fn flush(&self) -> Result<(), ServicesError> {
        let res = self.tcns_batch.lock();
        let mut tcns = expect_log!(res, "Couldn't lock tcns batch");

        let merged = self.merge_with_db(tcns.clone())?;
        self.update_db(merged)?;

        tcns.clear();

        Ok(())
    }

    fn merge_with_db(&self, tcns: Vec<ObservedTcn>) -> Result<Vec<ObservedTcn>, ServicesError> {
        let db_tcns = self
            .tcn_dao
            .find_tcns(tcns.clone().into_iter().map(|tcn| tcn.tcn).collect())?;

        let db_tcns_map: HashMap<[u8; 16], ObservedTcn> =
            db_tcns.into_iter().map(|tcn| (tcn.tcn.0, tcn)).collect();

        Ok(tcns
            .into_iter()
            .map(|tcn| match db_tcns_map.get(&tcn.tcn.0) {
                Some(db_tcn) => {
                    Self::merge_tcn_with_db(self.exposure_grouper.clone(), db_tcn.to_owned(), tcn)
                }
                None => tcn,
            })
            .collect())
    }

    // Assumes: tcn contact_start always after db_tcn contact_start
    fn merge_tcn_with_db(
        exposure_grouper: ExposureGrouper,
        db_tcn: ObservedTcn,
        tcn: ObservedTcn,
    ) -> ObservedTcn {
        if exposure_grouper.is_contiguous(&db_tcn, &tcn) {
            // Put db TCN and new TCN in an exposure as convenience to re-calculate measurements.
            let mut exposure = Exposure::create(db_tcn);
            exposure.push(tcn.clone());
            let measurements = exposure.measurements();
            ObservedTcn {
                tcn: tcn.tcn,
                contact_start: measurements.contact_start,
                contact_end: measurements.contact_end,
                min_distance: measurements.min_distance,
            }
        } else {
            tcn
        }
    }

    fn update_db(&self, tcns: Vec<ObservedTcn>) -> Result<(), ServicesError> {
        self.tcn_dao.save_batch(tcns)
    }

    pub fn new(tcn_dao: Arc<T>, exposure_grouper: ExposureGrouper) -> TcnBatchesManager<T> {
        TcnBatchesManager {
            tcn_dao,
            tcns_batch: Arc::new(Mutex::new(vec![])),
            exposure_grouper,
        }
    }

    pub fn push(&self, tcn: ObservedTcn) {
        let res = self.tcns_batch.lock();
        let mut tcns = expect_log!(res, "Couldn't lock tcns batch");
        tcns.push(tcn);
    }

    pub fn len(&self) -> Result<usize, ServicesError> {
        match self.tcns_batch.lock() {
            Ok(tcns) => Ok(tcns.len()),
            Err(_) => Err(ServicesError::General("Couldn't lock tcns batch".to_owned()))
        }
    }
}

pub struct ObservedTcnProcessorImpl<T>
where
    T: 'static + TcnDao,
{
    tcn_batches_manager: Arc<Mutex<TcnBatchesManager<T>>>,
    timer: Arc<Mutex<Timer>>,
}

impl<T> ObservedTcnProcessorImpl<T>
where
    T: 'static + TcnDao,
{
    pub fn new(tcn_batches_manager: TcnBatchesManager<T>) -> ObservedTcnProcessorImpl<T> {
        let instance = ObservedTcnProcessorImpl {
            tcn_batches_manager: Arc::new(Mutex::new(tcn_batches_manager)),
            timer: Arc::new(Mutex::new(Timer::new())),
        };
        instance.schedule_process_batches();
        instance
    }

    fn schedule_process_batches(&self) {
        let timer_mutex = self.timer.clone();
        let tcn_batches_manager = self.tcn_batches_manager.clone();
        thread::spawn(move || {
            let timer = timer_mutex.lock().unwrap();
            timer.schedule_repeating(chrono::Duration::seconds(10), move || {
                debug!("Flushing TCN batches into database");
                let tcn_batches_manager_res = tcn_batches_manager.lock();
                let tcn_batches_manager = expect_log!(tcn_batches_manager_res, "error");
                let flush_res = tcn_batches_manager.flush();
                expect_log!(flush_res, "Couldn't flush TCNs");
            })
        });
    }
}

impl<T> ObservedTcnProcessor for ObservedTcnProcessorImpl<T>
where
    T: TcnDao + Sync + Send,
{
    fn save(&self, tcn_str: &str, distance: f32) -> Result<(), ServicesError> {
        info!("Recording a TCN {:?}", tcn_str);

        let bytes_vec: Vec<u8> = hex::decode(tcn_str)?;
        let observed_tcn = ObservedTcn {
            tcn: TemporaryContactNumber(byte_vec_to_16_byte_array(bytes_vec)),
            contact_start: UnixTime { value: 0 },
            contact_end: UnixTime { value: 0 },
            min_distance: distance,
        };

        let res = self.tcn_batches_manager.lock();
        let tcns = expect_log!(res, "Couldn't lock tcns batch");
        tcns.push(observed_tcn);

        Ok(())
    }
}

pub trait TcnDao: Send + Sync {
    fn all(&self) -> Result<Vec<ObservedTcn>, ServicesError>;
    fn find_tcns(
        &self,
        with: Vec<TemporaryContactNumber>,
    ) -> Result<Vec<ObservedTcn>, ServicesError>;
    fn save_batch(&self, observed_tcns: Vec<ObservedTcn>) -> Result<(), ServicesError>;
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
                tcn text primary key,
                contact_start integer not null,
                contact_end integer not null,
                min_distance real not null
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

        ObservedTcn {
            tcn,
            contact_start: UnixTime {
                value: contact_start as u64,
            },
            contact_end: UnixTime {
                value: contact_end as u64,
            },
            min_distance: min_distance as f32,
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
                "select tcn, contact_start, contact_end, min_distance from tcn",
                NO_PARAMS,
                |row| Self::to_tcn(row),
            )
            .map_err(ServicesError::from)
    }

    fn find_tcns(
        &self,
        _with: Vec<TemporaryContactNumber>,
    ) -> Result<Vec<ObservedTcn>, ServicesError> {
        // TODO: IN query
        // let tcn_strs: Vec<String> = with.into_iter().map(|tcn| hex::encode(tcn.0)).collect();
        // self.db
        //     .query(
        //         "select tcn, contact_start, contact_end, min_distance from tcn where tcn in ?1",
        //         // params![tcn_strs],
        //         params![vec![foo]],
        //         |row| Self::to_tcn(row),
        //     )
        //     .map_err(ServicesError::from)

        self.all()
    }

    // Overwrites if already exists
    fn save_batch(&self, observed_tcns: Vec<ObservedTcn>) -> Result<(), ServicesError> {
        self.db.transaction(|t| {
            for tcn in observed_tcns {
                let tcn_str = hex::encode(tcn.tcn.0);
                let res = t.execute("insert or replace into tcn(tcn, contact_start, contact_end, min_distance) values(?1, ?2, ?3, ?4)",
                params![
                    tcn_str,
                    tcn.contact_start.value as i64,
                    tcn.contact_end.value as i64,
                    tcn.min_distance as f64 // db requires f64 / real
                ]);
                if res.is_err() {
                    return Err(ServicesError::General("Insert TCN failed".to_owned()))
                }
                
            }
            Ok(())
        })
    }
}

pub trait ByteArrayMappable {
    fn as_bytes(&self) -> [u8; 8];
}

impl ByteArrayMappable for u64 {
    // Returns u64 as little endian byte array
    fn as_bytes(&self) -> [u8; 8] {
        (0..8).fold([0; 8], |mut acc, index| {
            let value: u8 = ((self >> (index * 8)) & 0xFF) as u8;
            acc[index] = value;
            acc
        })
    }
}

// Note: this struct is meant only to send to the app, thus time directly as u64.
// Ideally these types would be separated (e.g. in an own module)
#[derive(Debug, Serialize)]
pub struct Alert {
    pub id: String,
    pub report: PublicReport,
    pub contact_start: u64,
    pub contact_end: u64,
    pub min_distance: f32,
}

mod exposure {
    use super::ObservedTcn;
    use crate::{errors::ServicesError, reports_interval::UnixTime};

    #[derive(PartialEq, Debug)]
    pub struct Exposure {
        // Can't be empty
        tcns: Vec<ObservedTcn>,
    }

    impl Exposure {
        pub fn create(tcn: ObservedTcn) -> Exposure {
            Exposure { tcns: vec![tcn] }
        }

        // Only used in tests
        #[allow(dead_code)]
        pub fn create_with_tcns(tcns: Vec<ObservedTcn>) -> Result<Exposure, ServicesError> {
            if tcns.is_empty() {
                Err(ServicesError::General(
                    "Exposure can't be created without TCNs.".to_owned(),
                ))
            } else {
                Ok(Exposure { tcns })
            }
        }

        pub fn push(&mut self, tcn: ObservedTcn) {
            self.tcns.push(tcn);
        }

        pub fn last(&self) -> ObservedTcn {
            // Unwrap: struct guarantees that tcns can't be empty.
            self.tcns.last().unwrap().clone()
        }

        pub fn measurements(&self) -> ExposureMeasurements {
            let mut tcns = self.tcns.clone();
            tcns.sort_by_key(|tcn| tcn.contact_start.value);

            let first_tcn = tcns
                .first()
                .expect("Invalid state: struct guarantees that tcns can't be empty");

            let contact_start = first_tcn.contact_start.value;
            let contact_end = tcns.last().unwrap_or(first_tcn).contact_end.value;
            let min_distance = tcns
                .into_iter()
                .fold(std::f32::MAX, |acc, el| f32::min(acc, el.min_distance));

            ExposureMeasurements {
                contact_start: UnixTime {
                    value: contact_start,
                },
                contact_end: UnixTime { value: contact_end },
                min_distance,
            }
        }
    }

    pub struct ExposureMeasurements {
        pub contact_start: UnixTime,
        pub contact_end: UnixTime,
        pub min_distance: f32,
    }
}

// Groups TCNs by contiguity.
#[derive(Clone)]
pub struct ExposureGrouper {
    pub threshold: u64,
}

impl ExposureGrouper {
    fn group(&self, mut tcns: Vec<ObservedTcn>) -> Vec<Exposure> {
        tcns.sort_by_key(|tcn| tcn.contact_start.value);

        let mut exposures: Vec<Exposure> = vec![];
        for tcn in tcns {
            match exposures.last_mut() {
                Some(last_group) => {
                    if self.is_contiguous(&last_group.last(), &tcn) {
                        last_group.push(tcn)
                    } else {
                        exposures.push(Exposure::create(tcn));
                    }
                }
                None => exposures.push(Exposure::create(tcn)),
            }
        }
        exposures
    }

    // Notes: 
    // - Expects tcn2.start > tcn1.start. If will return otherwise always true.
    // - Overlapping is considered contiguous.
    // (Note that depending on the implementation of writes, overlaps may not be possible.)
    fn is_contiguous(&self, tcn1: &ObservedTcn, tcn2: &ObservedTcn) -> bool {
        // Signed: overlap (start2 < end1) considered contiguous.
        (tcn2.contact_start.value as i64 - tcn1.contact_end.value as i64) < self.threshold as i64
    }
}

trait SignedReportExt {
    fn with_str(str: &str) -> Option<SignedReport> {
        base64::decode(str)
            .also(|res| {
                if let Err(error) = res {
                    error!("Error: {} decoding (base64) report: {:?}", error, res)
                }
            })
            .map_err(Error::from)
            .and_then(|bytes| SignedReport::read(bytes.as_slice()).map_err(Error::from))
            .map_err(|err| {
                error!("Error decoding or generating report: {}", err);
                err
            })
            .ok()
    }
}
impl SignedReportExt for SignedReport {}

pub struct ReportsUpdater<'a, T: Preferences, U: TcnDao, V: TcnMatcher, W: TcnApi, X: MemoMapper> {
    pub preferences: Arc<T>,
    pub tcn_dao: Arc<U>,
    pub tcn_matcher: V,
    pub api: &'a W,
    pub memo_mapper: &'a X,
    pub exposure_grouper: ExposureGrouper,
}

impl<'a, T, U, V, W, X> ReportsUpdater<'a, T, U, V, W, X>
where
    T: Preferences,
    U: TcnDao,
    V: TcnMatcher,
    W: TcnApi,
    X: MemoMapper,
{
    pub fn fetch_new_reports(&self) -> Result<Vec<Alert>, ServicesError> {
        self.retrieve_and_match_new_reports().map(|signed_reports| {
            signed_reports
                .into_iter()
                .filter_map(|matched_report| self.to_ffi_alerts(matched_report).ok())
                .flatten()
                .collect()
        })
    }

    // Note: For now we will not create an FFI layer to handle JSON conversions, since it may be possible
    // to use directly the data structures.
    fn to_ffi_alerts(&self, matched_report: MatchedReport) -> Result<Vec<Alert>, ServicesError> {
        let exposures = self.exposure_grouper.group(matched_report.clone().tcns);

        exposures
            .into_iter()
            .map(|exposure_tcns| self.to_alert(matched_report.report.clone(), exposure_tcns))
            .collect()
    }

    fn to_alert(
        &self,
        signed_report: SignedReport,
        exposure: Exposure,
    ) -> Result<Alert, ServicesError> {
        let report = signed_report.clone().verify()?;

        let public_report = self.memo_mapper.to_report(Memo {
            bytes: report.memo_data().to_vec(),
        });

        let measurements = exposure.measurements();

        Ok(Alert {
            id: format!("{:?}", signed_report.sig),
            report: public_report,
            contact_start: measurements.contact_start.value,
            contact_end: measurements.contact_end.value,
            min_distance: measurements.min_distance,
        })
    }

    fn retrieve_and_match_new_reports(&self) -> Result<Vec<MatchedReport>, ServicesError> {
        let now: UnixTime = UnixTime::now();

        let matching_reports = self.matching_reports(self.determine_start_interval(&now), &now);

        if let Ok(matching_reports) = &matching_reports {
            let intervals = matching_reports.iter().map(|c| c.interval).collect();
            self.store_last_completed_interval(intervals, &now);
        };

        matching_reports
            .map(|chunks| chunks.into_iter().flat_map(|chunk| chunk.matched).collect())
            .map_err(ServicesError::from)
    }

    fn retrieve_last_completed_interval(&self) -> Option<ReportsInterval> {
        self.preferences.last_completed_reports_interval()
    }

    fn determine_start_interval(&self, time: &UnixTime) -> ReportsInterval {
        let last = self.retrieve_last_completed_interval();
        debug!(
            "Determining start reports interval. Last completed interval: {:?}",
            last
        );
        let next = last.map(|interval| interval.next());
        debug!("Next interval: {:?}", next);
        let result = next.unwrap_or_else(|| ReportsInterval::create_for_with_default_length(time));
        debug!("Interval to fetch: {:?}", result);
        result
    }

    fn matching_reports(
        &self,
        start_interval: ReportsInterval,
        until: &UnixTime,
    ) -> Result<Vec<MatchedReportsChunk>, ServicesError> {
        let sequence = Self::generate_intervals_sequence(start_interval, until);
        let reports = sequence.map(|interval| self.retrieve_reports(interval));
        let matched_results = reports.map(|interval| self.match_retrieved_reports_result(interval));
        matched_results
            .into_iter()
            .collect::<Result<Vec<MatchedReportsChunk>, ServicesError>>()
            .map_err(ServicesError::from)
    }

    fn generate_intervals_sequence(
        from: ReportsInterval,
        until: &UnixTime,
    ) -> impl Iterator<Item = ReportsInterval> + '_ {
        std::iter::successors(Some(from), |item| Some(item.next()))
            .take_while(move |item| item.starts_before(until))
    }

    fn retrieve_reports(
        &self,
        interval: ReportsInterval,
    ) -> Result<SignedReportsChunk, NetworkingError> {
        let reports_strings_result: Result<Vec<String>, NetworkingError> =
            self.api.get_reports(interval.number, interval.length);

        reports_strings_result.map(|report_strings| SignedReportsChunk {
            reports: report_strings
                .into_iter()
                .filter_map(|report_string| {
                    SignedReport::with_str(&report_string).also(|res| {
                        if res.is_none() {
                            error!("Failed to convert report string: $it to report");
                        }
                    })
                })
                .collect(),
            interval,
        })
    }

    fn match_retrieved_reports_result(
        &self,
        reports_result: Result<SignedReportsChunk, NetworkingError>,
    ) -> Result<MatchedReportsChunk, ServicesError> {
        reports_result
            .map_err(ServicesError::from)
            .and_then(|chunk| self.to_matched_reports_chunk(&chunk))
    }

    /**
     * Maps reports chunk to a new chunk containing possible matches.
     */
    fn to_matched_reports_chunk(
        &self,
        chunk: &SignedReportsChunk,
    ) -> Result<MatchedReportsChunk, ServicesError> {
        self.find_matches(chunk.reports.clone())
            .map(|matches| MatchedReportsChunk {
                reports: chunk.reports.clone(),
                matched: matches,
                interval: chunk.interval.clone(),
            })
            .map_err(ServicesError::from)
    }

    fn find_matches(
        &self,
        reports: Vec<SignedReport>,
    ) -> Result<Vec<MatchedReport>, ServicesError> {
        let matching_start_time = Instant::now();

        info!("R Start matching...");

        let tcns = self.tcn_dao.all();

        if let Ok(tcns) = &tcns {
            let tcns_for_debugging: Vec<String> = tcns
                .clone()
                .into_iter()
                .map(|tcn| hex::encode(tcn.tcn.0))
                .collect();

            info!("R DB TCNs: {:?}", tcns_for_debugging);
        }

        let matched_reports: Result<Vec<MatchedReport>, ServicesError> =
            tcns.and_then(|tcns| self.tcn_matcher.match_reports(tcns, reports));

        let time = matching_start_time.elapsed().as_secs();
        info!("Took {:?}s to match reports", time);

        if let Ok(reports) = &matched_reports {
            if !reports.is_empty() {
                let reports_strings: Vec<String> = reports
                    .into_iter()
                    .map(|report| base64::encode(signed_report_to_bytes(report.report.clone())))
                    .collect();

                info!("Matches found ({:?}): {:?}", reports.len(), reports_strings);
            } else {
                info!("No matches found");
            }
        };

        if let Err(error) = &matched_reports {
            error!("Matching error: ({:?})", error)
        }

        matched_reports
    }

    fn interval_ending_before(
        intervals: Vec<ReportsInterval>,
        time: &UnixTime,
    ) -> Option<ReportsInterval> {
        // TODO shorter version of this?
        let reversed: Vec<ReportsInterval> = intervals.into_iter().rev().collect();
        reversed.into_iter().find(|i| i.ends_before(&time))
    }

    fn store_last_completed_interval(&self, intervals: Vec<ReportsInterval>, now: &UnixTime) {
        let interval = Self::interval_ending_before(intervals.clone(), now);
        debug!(
            "Storing last completed reports interval: {:?}, for intervals: {:?}",
            interval, intervals
        );

        if let Some(interval) = interval {
            self.preferences
                .set_last_completed_reports_interval(interval);
        }
    }
}

// To insert easily side effects in flows anywhere (from Kotlin)
trait Also: Sized {
    fn also<T>(self, f: T) -> Self
    where
        T: FnOnce(&Self) -> (),
    {
        f(&self);
        self
    }
}

impl<T> Also for T {}

#[derive(Debug, Clone)]
struct MatchedReportsChunk {
    reports: Vec<SignedReport>,
    matched: Vec<MatchedReport>,
    interval: ReportsInterval,
}

#[derive(Debug, Clone)]
struct SignedReportsChunk {
    reports: Vec<SignedReport>,
    interval: ReportsInterval,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        networking::TcnApiImpl,
        preferences::PreferencesImpl,
        reporting::{
            memo::MemoMapperImpl,
            public_report::{CoughSeverity, FeverSeverity},
            symptom_inputs::UserInput,
        },
    };
    use rusqlite::Connection;
    use tcn::{MemoType, ReportAuthorizationKey};

    #[test]
    fn interval_ending_before_if_contained_and_one_interval() {
        let containing_interval = ReportsInterval {
            number: 73690,
            length: 21600,
        };

        let intervals: Vec<ReportsInterval> = vec![containing_interval];

        let time = UnixTime {
            value: containing_interval.start() + 2000, // Arbitrary value inside interval
        };

        let interval_ending_before: Option<ReportsInterval> =
            ReportsUpdater::<
                'static,
                PreferencesImpl,
                TcnDaoImpl,
                TcnMatcherRayon,
                TcnApiImpl,
                MemoMapperImpl,
            >::interval_ending_before(intervals, &time);

        // time is contained in the interval, and it's the only interval, so there's no interval ending before of time's interval
        assert!(interval_ending_before.is_none());
    }

    #[test]
    fn interval_ending_before_if_there_is_one() {
        let containing_interval = ReportsInterval {
            number: 73690,
            length: 21600,
        };

        let interval_before = ReportsInterval {
            number: containing_interval.number - 1,
            length: 21600,
        };

        let intervals: Vec<ReportsInterval> = vec![interval_before, containing_interval];

        let time = UnixTime {
            value: containing_interval.start() + 2000, // Arbitrary value inside interval
        };

        let interval_ending_before: Option<ReportsInterval> =
            ReportsUpdater::<
                'static,
                PreferencesImpl,
                TcnDaoImpl,
                TcnMatcherRayon,
                TcnApiImpl,
                MemoMapperImpl,
            >::interval_ending_before(intervals, &time);

        assert!(interval_ending_before.is_some());
        assert_eq!(interval_ending_before.unwrap(), interval_before);
    }

    #[test]
    fn interval_ending_before_is_none_if_empty() {
        let intervals: Vec<ReportsInterval> = vec![];

        let time = UnixTime { value: 1591706000 }; // arbitrary time

        let interval_ending_before: Option<ReportsInterval> =
            ReportsUpdater::<
                'static,
                PreferencesImpl,
                TcnDaoImpl,
                TcnMatcherRayon,
                TcnApiImpl,
                MemoMapperImpl,
            >::interval_ending_before(intervals, &time);

        assert!(interval_ending_before.is_none());
    }

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
        };

        let save_res = tcn_dao.save_batch(vec![observed_tcn.clone()]);
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
        };
        let observed_tcn_2 = ObservedTcn {
            tcn: TemporaryContactNumber([
                43, 229, 125, 245, 98, 86, 100, 1, 172, 25, 0, 150, 123, 66, 34, 12,
            ]),
            contact_start: UnixTime { value: 1590518190 },
            contact_end: UnixTime { value: 1590518191 },
            min_distance: 0.0,
        };
        let observed_tcn_3 = ObservedTcn {
            tcn: TemporaryContactNumber([
                11, 246, 125, 123, 102, 86, 100, 1, 34, 25, 21, 150, 99, 66, 34, 0,
            ]),
            contact_start: UnixTime { value: 2230522104 },
            contact_end: UnixTime { value: 2230522105 },
            min_distance: 0.0,
        };

        let save_res_1 = tcn_dao.save_batch(vec![observed_tcn_1.clone()]);
        let save_res_2 = tcn_dao.save_batch(vec![observed_tcn_2.clone()]);
        let save_res_3 = tcn_dao.save_batch(vec![observed_tcn_3.clone()]);
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

    // Utility to see quickly all TCNs (hex) for a report
    #[test]
    #[ignore]
    fn print_tcns_for_report() {
        let report_str = "rOFMgzy3y36MJns34Xj7EZu5Dti9XMhYGRpa/DVznep6q4hMtMYm9sYMg9+sRSHAj0Ff2rHTPXskuzJH0+pZMQEAAgAAFAEAnazaXgAAAAD//////////wMAMFLrKLNOvwUJQSNta9rlzTyjFdpfq25Kv34c6y+ZOoSzRewzNAWsd56Yzm8LUw9cpHB8yyzDUMJ9YTKhD8dADA==";
        let report = SignedReport::with_str(report_str).unwrap();
        info!("{:?}", report);
        for tcn in report.verify().unwrap().temporary_contact_numbers() {
            info!("{}", hex::encode(tcn.0));
        }
    }

    #[test]
    fn one_report_matches() {
        let verification_report_str = "D7Z8XrufMgfsFH3K5COnv17IFG2ahDb4VM/UMK/5y0+/OtUVVTh7sN0DQ5+R+ocecTilR+SIIpPHzujeJdJzugEAECcAFAEAmmq5XgAAAACaarleAAAAACEBo8p1WdGeXb5O5/3kN6x7GSylgiYGIGsABl3NrxhJu9XHwsN3f6yvRwUxs2fhP4oU5E3+JWabBP6v09pGV1xRCw==";
        let verification_report_tcn: [u8; 16] = [
            24, 229, 125, 245, 98, 86, 219, 221, 172, 25, 232, 150, 206, 66, 164, 173,
        ]; // belongs to report
        let verification_contact_start = UnixTime { value: 1590528300 };
        let verification_contact_end = UnixTime { value: 1590528301 };
        let verification_min_distance = 2.3;
        let verification_report = SignedReport::with_str(verification_report_str).unwrap();

        let mut reports: Vec<SignedReport> = vec![0; 20]
            .into_iter()
            .map(|_| create_test_report())
            .collect();
        reports.push(verification_report);

        // let matcher = TcnMatcherStdThreadSpawn {}; // 20 -> 1s, 200 -> 16s, 1000 -> 84s, 10000 ->
        let matcher = TcnMatcherRayon {}; // 20 -> 1s, 200 -> 7s, 1000 -> 87s, 10000 -> 927s

        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber(verification_report_tcn),
                contact_start: verification_contact_start.clone(),
                contact_end: verification_contact_end.clone(),
                min_distance: verification_min_distance,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
            },
        ];

        let res = matcher.match_reports(tcns, reports);
        let matches = res.unwrap();
        assert_eq!(matches.len(), 1);

        let matched_report_str = base64::encode(signed_report_to_bytes(matches[0].report.clone()));
        assert_eq!(matched_report_str, verification_report_str);
        assert_eq!(matches[0].tcns[0].contact_start, verification_contact_start);
        assert_eq!(matches[0].tcns[0].contact_end, verification_contact_end);
        assert_eq!(matches[0].tcns[0].min_distance, verification_min_distance);
    }

    #[test]
    #[ignore]
    fn matching_benchmark() {
        let verification_report_str = "D7Z8XrufMgfsFH3K5COnv17IFG2ahDb4VM/UMK/5y0+/OtUVVTh7sN0DQ5+R+ocecTilR+SIIpPHzujeJdJzugEAECcAFAEAmmq5XgAAAACaarleAAAAACEBo8p1WdGeXb5O5/3kN6x7GSylgiYGIGsABl3NrxhJu9XHwsN3f6yvRwUxs2fhP4oU5E3+JWabBP6v09pGV1xRCw==";
        let verification_report_tcn: [u8; 16] = [
            24, 229, 125, 245, 98, 86, 219, 221, 172, 25, 232, 150, 206, 66, 164, 173,
        ]; // belongs to report
        let verification_contact_time = UnixTime { value: 1590528300 };
        let verification_report = SignedReport::with_str(verification_report_str).unwrap();

        let mut reports: Vec<SignedReport> = vec![0; 20]
            .into_iter()
            .map(|_| create_test_report())
            .collect();
        reports.push(verification_report);

        // let matcher = TcnMatcherStdThreadSpawn {}; // 20 -> 1s, 200 -> 16s, 1000 -> 84s, 10000 ->
        let matcher = TcnMatcherRayon {}; // 20 -> 1s, 200 -> 7s, 1000 -> 87s, 10000 -> 927s

        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber(verification_report_tcn),
                contact_start: verification_contact_time.clone(),
                contact_end: verification_contact_time.clone(),
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
            },
        ];

        let matching_start_time = Instant::now();

        let res = matcher.match_reports(tcns, reports);

        let matches = res.unwrap();
        assert_eq!(matches.len(), 1);

        let time = matching_start_time.elapsed().as_secs();
        info!("Took {:?}s to match reports", time);

        // Short verification that matching is working
        let matched_report_str = base64::encode(signed_report_to_bytes(matches[0].report.clone()));
        assert_eq!(matched_report_str, verification_report_str);
    }

    #[test]
    fn test_report_empty_is_none() {
        assert!(SignedReport::with_str("").is_none())
    }

    #[test]
    fn test_report_base64_invalid_is_none() {
        assert!(SignedReport::with_str("%~=-🥳").is_none())
    }

    #[test]
    fn test_report_base64_valid_report_invalid_is_none() {
        assert!(SignedReport::with_str("slkdjfslfd").is_none())
    }

    #[test]
    fn test_group_in_exposures_empty() {
        let tcns = vec![];
        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_group_in_exposures_same_group() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1500 },
                contact_end: UnixTime { value: 1501 },
                min_distance: 0.0,
            },
        ];
        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], Exposure::create_with_tcns(tcns).unwrap());
    }

    #[test]
    fn test_group_in_exposures_identical_tcns() {
        // Passing same TCN 2x (normally will not happen)
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
            },
        ];

        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], Exposure::create_with_tcns(tcns).unwrap());
    }

    #[test]
    fn test_group_in_exposures_different_groups() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 2002 },
                contact_end: UnixTime { value: 2501 },
                min_distance: 0.0,
            },
        ];

        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups[0],
            Exposure::create_with_tcns(vec![tcns[0].clone()]).unwrap()
        );
        assert_eq!(
            groups[1],
            Exposure::create_with_tcns(vec![tcns[1].clone()]).unwrap()
        );
    }

    #[test]
    fn test_group_in_exposures_sort() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 2002 },
                contact_end: UnixTime { value: 2501 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
            },
        ];

        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups[0],
            Exposure::create_with_tcns(vec![tcns[1].clone()]).unwrap()
        );
        assert_eq!(
            groups[1],
            Exposure::create_with_tcns(vec![tcns[0].clone()]).unwrap()
        );
    }

    #[test]
    fn test_group_in_exposures_overlap() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 2000 },
                min_distance: 0.0,
            },
            // starts before previous TCN ends
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 2600 },
                min_distance: 0.0,
            },
        ];

        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], Exposure::create_with_tcns(tcns).unwrap());
    }

    #[test]
    fn test_group_in_exposures_mixed() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 3000 },
                contact_end: UnixTime { value: 3001 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1 },
                contact_end: UnixTime { value: 2 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 3900 },
                contact_end: UnixTime { value: 4500 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 500 },
                contact_end: UnixTime { value: 501 },
                min_distance: 0.0,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1589209754 },
                contact_end: UnixTime { value: 1589209755 },
                min_distance: 0.0,
            },
        ];

        let groups = ExposureGrouper { threshold: 1000 }.group(tcns.clone());

        assert_eq!(groups.len(), 3);
        assert_eq!(
            groups[0],
            Exposure::create_with_tcns(vec![tcns[1].clone(), tcns[3].clone()]).unwrap()
        );
        assert_eq!(
            groups[1],
            Exposure::create_with_tcns(vec![tcns[0].clone(), tcns[2].clone()]).unwrap()
        );
        assert_eq!(
            groups[2],
            Exposure::create_with_tcns(vec![tcns[4].clone().clone()]).unwrap()
        );
    }

    #[test]
    fn test_exposure_measurements_correct() {
        let tcns = vec![
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 2600 },
                min_distance: 2.3,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 2601 },
                contact_end: UnixTime { value: 3223 },
                min_distance: 0.845,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 2000 },

                min_distance: 0.846,
            },
        ];

        let measurements = Exposure::create_with_tcns(tcns).unwrap().measurements();

        assert_eq!(measurements.contact_start.value, 1000);
        assert_eq!(measurements.contact_end.value, 3223);
        assert_eq!(measurements.min_distance, 0.845);
    }

    #[test]
    fn test_update_overwrites_tcn() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database.clone());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        };

        let res = tcn_dao.save_batch(vec![tcn.clone()]);

        assert!(res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(loaded_tcns.len(), 1);
        assert_eq!(loaded_tcns[0], tcn);

        let new_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1000 },
            contact_end: UnixTime { value: 2000 },
            min_distance: 1.21,
        };
        let res = tcn_dao.save_batch(vec![new_tcn.clone()]);
        assert!(res.is_ok());

        let loaded_tcns_res_new = tcn_dao.all();
        assert!(loaded_tcns_res_new.is_ok());

        let loaded_tcns_new = loaded_tcns_res_new.unwrap();
        assert_eq!(loaded_tcns_new.len(), 1);
        assert_eq!(loaded_tcns_new[0].tcn, new_tcn.tcn);
        assert_eq!(loaded_tcns_new[0].contact_start.value, 1000);
        assert_eq!(loaded_tcns_new[0].contact_end.value, 2000);
        assert_eq!(loaded_tcns_new[0].min_distance, 1.21);
    }

    #[test]
    fn test_push_adds_tcns_to_batch_manager() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database.clone());

        let batches_manager = TcnBatchesManager::new(Arc::new(tcn_dao), ExposureGrouper{ threshold: 1000});

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        });

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        });

        let len_res = batches_manager.len();
        assert!(len_res.is_ok());
        assert_eq!(2, len_res.unwrap())
    }

    #[test]
    fn test_flush_clears_tcns() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = TcnDaoImpl::new(database.clone());

        let batches_manager = TcnBatchesManager::new(Arc::new(tcn_dao), ExposureGrouper{ threshold: 1000});

        batches_manager.push(ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
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
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let batches_manager = TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper{ threshold: 1000});

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
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
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let batches_manager = TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper{ threshold: 1000});

        let stored_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        };
        let save_res = tcn_dao.save_batch(vec![stored_tcn]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
        };
        batches_manager.push(tcn.clone());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(1, loaded_tcns.len());

        assert_eq!(loaded_tcns[0], ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
        });
    }

    #[test]
    fn test_flush_does_not_affect_different_stored_tcn() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let batches_manager = TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper{ threshold: 1000});

        let stored_tcn = ObservedTcn {
            tcn: TemporaryContactNumber([1; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        };
        let save_res = tcn_dao.save_batch(vec![stored_tcn]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
        };
        batches_manager.push(tcn.clone());

        let flush_res = batches_manager.flush();
        assert!(flush_res.is_ok());

        let loaded_tcns_res = tcn_dao.all();
        assert!(loaded_tcns_res.is_ok());

        let loaded_tcns = loaded_tcns_res.unwrap();
        assert_eq!(2, loaded_tcns.len());

        assert_eq!(loaded_tcns[0], ObservedTcn {
            tcn: TemporaryContactNumber([1; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        });
        assert_eq!(loaded_tcns[1], ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 5000 },
            min_distance: 1.12,
        });
    }

    #[test]
    fn test_flush_updates_correctly_2_stored_1_updated() {
        let database = Arc::new(Database::new(
            Connection::open_in_memory().expect("Couldn't create database!"),
        ));
        let tcn_dao = Arc::new(TcnDaoImpl::new(database.clone()));

        let batches_manager = TcnBatchesManager::new(tcn_dao.clone(), ExposureGrouper{ threshold: 1000});

        let stored_tcn1 = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1000 },
            contact_end: UnixTime { value: 6000 },
            min_distance: 0.4,
        };

        let stored_tcn2 = ObservedTcn {
            tcn: TemporaryContactNumber([1; 16]),
            contact_start: UnixTime { value: 1600 },
            contact_end: UnixTime { value: 2600 },
            min_distance: 2.3,
        };
        let save_res = tcn_dao.save_batch(vec![stored_tcn1.clone(), stored_tcn2.clone()]);
        assert!(save_res.is_ok());

        let tcn = ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 3000 },
            contact_end: UnixTime { value: 7000 },
            min_distance: 1.12,
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

        assert_eq!(loaded_tcns[0], ObservedTcn {
            tcn: TemporaryContactNumber([0; 16]),
            contact_start: UnixTime { value: 1000 },
            contact_end: UnixTime { value: 7000 },
            min_distance: 0.4,
        });
        assert_eq!(loaded_tcns[1], stored_tcn2);
    }


    fn create_test_report() -> SignedReport {
        let memo_mapper = MemoMapperImpl {};
        let public_report = PublicReport {
            report_time: UnixTime { value: 1589209754 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1589209754 }),
            fever_severity: FeverSeverity::Serious,
            cough_severity: CoughSeverity::Existing,
            breathlessness: true,
            muscle_aches: true,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: true,
            other: false,
            no_symptoms: true,
        };
        let rak = ReportAuthorizationKey::new(rand::thread_rng());
        let memo_data = memo_mapper.to_memo(public_report);
        rak.create_report(MemoType::CoEpiV1, memo_data.bytes, 1, 10000)
            .unwrap()
    }
}

// Testing / debugging
fn signed_report_to_bytes(signed_report: SignedReport) -> Vec<u8> {
    let mut buf = Vec::new();
    signed_report
        .write(Cursor::new(&mut buf))
        .expect("Couldn't write signed report bytes");
    buf
}
