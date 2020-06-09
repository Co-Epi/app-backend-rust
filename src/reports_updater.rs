use crate::{
    byte_vec_to_16_byte_array, byte_vec_to_24_byte_array, byte_vec_to_8_byte_array,
    errors::{Error, ServicesError},
    networking::{NetworkingError, TcnApi},
    preferences::{Preferences, PreferencesKey},
    reporting::{
        memo::{Memo, MemoMapper},
        public_report::PublicReport,
    },
    reports_interval, DB, DB_UNINIT,
};
use chrono::Utc;
use log::*;
use rayon::prelude::*;
use reports_interval::{ReportsInterval, UnixTime};
use serde::Serialize;
use std::collections::HashMap;
use std::{sync::Arc, time::Instant};
use tcn::{SignedReport, TemporaryContactNumber};

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
    contact_time: UnixTime,
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
                let mut out: Option<MatchedReport> = None;
                for tcn in rep.temporary_contact_numbers() {
                    if let Some(entry) = observed_tcns_map.get(&tcn.0) {
                        out = Some(MatchedReport {
                            report: report.clone(),
                            contact_time: entry.time.clone(),
                        });
                        break;
                    }
                }
                out
            }
            Err(error) => {
                error!("Report can't be matched. Verification failed: {:?}", error);
                None
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ObservedTcn {
    tcn: TemporaryContactNumber,
    time: UnixTime,
}

impl ObservedTcn {
    fn as_bytes(&self) -> [u8; 24] {
        let tcn_bytes: [u8; 16] = self.tcn.0;
        let time_bytes: [u8; 8] = self.time.value.as_bytes();
        let total_bytes = [&tcn_bytes[..], &time_bytes[..]].concat();
        byte_vec_to_24_byte_array(total_bytes)
    }

    fn from_bytes(bytes: [u8; 24]) -> ObservedTcn {
        let tcn_bytes: [u8; 16] = byte_vec_to_16_byte_array(bytes[0..16].to_vec());
        let time_bytes: [u8; 8] = byte_vec_to_8_byte_array(bytes[16..24].to_vec());
        let time = u64::from_le_bytes(time_bytes);

        ObservedTcn {
            tcn: TemporaryContactNumber(tcn_bytes),
            time: UnixTime { value: time },
        }
    }
}

pub trait ObservedTcnProcessor {
    fn save(&self, tcn_str: &str) -> Result<(), ServicesError>;
}

pub struct ObservedTcnProcessorImpl<'a, T>
where
    T: TcnDao,
{
    pub tcn_dao: &'a T,
}

impl<'a, T> ObservedTcnProcessor for ObservedTcnProcessorImpl<'a, T>
where
    T: TcnDao,
{
    fn save(&self, tcn_str: &str) -> Result<(), ServicesError> {
        let bytes_vec: Vec<u8> = hex::decode(tcn_str)?;
        let observed_tcn = ObservedTcn {
            tcn: TemporaryContactNumber(byte_vec_to_16_byte_array(bytes_vec)),
            time: UnixTime {
                value: Utc::now().timestamp() as u64,
            },
        };
        self.tcn_dao.save(observed_tcn)
    }
}

pub trait TcnDao {
    fn all(&self) -> Result<Vec<ObservedTcn>, ServicesError>;
    fn save(&self, observed_tcn: ObservedTcn) -> Result<(), ServicesError>;
}

pub struct TcnDaoImpl {}
impl TcnDao for TcnDaoImpl {
    fn all(&self) -> Result<Vec<ObservedTcn>, ServicesError> {
        let mut out: Vec<ObservedTcn> = Vec::new();

        let items = DB
            .get()
            .ok_or(DB_UNINIT)
            .map_err(Error::from)?
            .scan("tcn")
            .map_err(Error::from)?;

        for (_id, content) in items {
            let byte_array: [u8; 24] = byte_vec_to_24_byte_array(content);
            out.push(ObservedTcn::from_bytes(byte_array));
        }

        Ok(out)
    }

    fn save(&self, observed_tcn: ObservedTcn) -> Result<(), ServicesError> {
        let db = DB.get().ok_or(DB_UNINIT)?;
        let mut tx = db.begin()?;

        tx.insert_record("tcn", &observed_tcn.as_bytes())?;
        // tx.put(CENS_BY_TS, ts, u128_of_tcn(tcn))?;
        tx.prepare_commit()?.commit()?;
        Ok(())
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
    id: String,
    report: PublicReport,
    contact_time: u64,
}

pub struct ReportsUpdater<'a, T: Preferences, U: TcnDao, V: TcnMatcher, W: TcnApi, X: MemoMapper> {
    pub preferences: Arc<T>,
    pub tcn_dao: &'a U,
    pub tcn_matcher: V,
    pub api: &'a W,
    pub memo_mapper: &'a X,
}

trait SignedReportExt {
    fn with_str(str: &str) -> Option<SignedReport> {
        base64::decode(str)
            .map_err(Error::from)
            .and_then(|bytes| SignedReport::read(bytes.as_slice()).map_err(Error::from))
            .also(|res| {
                if res.is_err() {
                    print!("error!");
                }
            })
            .map_err(|err| {
                error!("Error: {}", err);
                err
            })
            .ok()
    }
}
impl SignedReportExt for SignedReport {}

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
                .filter_map(|matched_report| self.to_ffi_alert(matched_report).ok())
                .collect()
        })
    }

    // Note: For now we will not create an FFI layer to handle JSON conversions, since it may be possible
    // to use directly the data structures.
    fn to_ffi_alert(&self, matched_report: MatchedReport) -> Result<Alert, ServicesError> {
        let report = matched_report.report.clone().verify()?;

        let public_report = self.memo_mapper.to_report(Memo {
            bytes: report.memo_data().to_vec(),
        });

        Ok(Alert {
            id: format!("{:?}", matched_report.report.sig),
            report: public_report,
            contact_time: matched_report.contact_time.value,
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
        self.preferences
            .last_completed_reports_interval(PreferencesKey::LastCompletedReportsInterval)
    }

    fn determine_start_interval(&self, time: &UnixTime) -> ReportsInterval {
        self.retrieve_last_completed_interval()
            .map(|interval| interval.next())
            .unwrap_or_else(|| ReportsInterval::create_for_with_default_length(time))
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
        info!("R DB TCNs count: {:?}", tcns);

        let matched_reports: Result<Vec<MatchedReport>, ServicesError> =
            tcns.and_then(|tcns| self.tcn_matcher.match_reports(tcns, reports));

        let time = matching_start_time.elapsed().as_secs();
        info!("Took {:?}s to match reports", time);

        if let Ok(reports) = &matched_reports {
            if !reports.is_empty() {
                info!("Matches found ({:?}): {:?}", reports.len(), matched_reports);
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
        let interval = Self::interval_ending_before(intervals, now);

        if let Some(interval) = interval {
            self.preferences.set_last_completed_reports_interval(
                PreferencesKey::LastCompletedReportsInterval,
                interval,
            );
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
    use crate::reporting::{
        memo::MemoMapperImpl,
        public_report::{CoughSeverity, FeverSeverity},
        symptom_inputs::UserInput,
    };
    use std::io::Cursor;
    use tcn::{MemoType, ReportAuthorizationKey};

    #[test]
    fn tcn_saved_and_restored_from_bytes() {
        let mut tcn_bytes: [u8; 16] = [0; 16];
        tcn_bytes[1] = 1;
        tcn_bytes[12] = 5;
        tcn_bytes[15] = 250;
        let time = UnixTime { value: 1590528300 };
        let observed_tcn = ObservedTcn {
            tcn: TemporaryContactNumber(tcn_bytes),
            time,
        };
        let observed_tcn_as_bytes = observed_tcn.as_bytes();
        let observed_tc_from_bytes = ObservedTcn::from_bytes(observed_tcn_as_bytes);
        assert_eq!(observed_tc_from_bytes, observed_tcn);
    }

    // Utility to see quickly all TCNs (hex) for a report
    // #[test]
    fn _print_tcns_for_report() {
        let report_str = "rOFMgzy3y36MJns34Xj7EZu5Dti9XMhYGRpa/DVznep6q4hMtMYm9sYMg9+sRSHAj0Ff2rHTPXskuzJH0+pZMQEAAgAAFAEAnazaXgAAAAD//////////wMAMFLrKLNOvwUJQSNta9rlzTyjFdpfq25Kv34c6y+ZOoSzRewzNAWsd56Yzm8LUw9cpHB8yyzDUMJ9YTKhD8dADA==";
        let report = SignedReport::with_str(report_str).unwrap();
        info!("{:?}", report);
        for tcn in report.verify().unwrap().temporary_contact_numbers() {
            info!("{}", hex::encode(tcn.0));
        }
    }

    #[test]
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
                time: UnixTime { value: 1590528300 },
            },
            ObservedTcn {
                tcn: TemporaryContactNumber(verification_report_tcn),
                time: verification_contact_time.clone(),
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                time: UnixTime { value: 1590528300 },
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
        assert_eq!(matches[0].contact_time, verification_contact_time);
    }

    fn create_test_report() -> SignedReport {
        let memo_mapper = MemoMapperImpl {};
        let public_report = PublicReport {
            report_time: UnixTime { value: 1589209754 },
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1589209754 }),
            fever_severity: FeverSeverity::Serious,
            breathlessness: true,
            cough_severity: CoughSeverity::Existing,
        };
        let rak = ReportAuthorizationKey::new(rand::thread_rng());
        let memo_data = memo_mapper.to_memo(public_report);
        rak.create_report(MemoType::CoEpiV1, memo_data.bytes, 1, 10000)
            .unwrap()
    }

    fn signed_report_to_bytes(signed_report: SignedReport) -> Vec<u8> {
        let mut buf = Vec::new();
        signed_report
            .write(Cursor::new(&mut buf))
            .expect("Couldn't write signed report bytes");
        buf
    }
}
