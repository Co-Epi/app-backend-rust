use super::{
    exposure::{Exposure, ExposureGrouper},
    tcn_matcher::{MatchedReport, TcnMatcher},
};
use crate::{
    database::{alert_dao::AlertDao, preferences::Preferences, tcn_dao::TcnDao},
    errors::{Error, ServicesError},
    extensions::Also,
    networking::{NetworkingError, TcnApi},
    reporting::{
        memo::{Memo, MemoMapper},
        public_report::PublicReport,
    },
    reports_interval, signed_report_to_bytes,
};
use log::*;
use reports_interval::{ReportsInterval, UnixTime};
use serde::Serialize;
use std::{sync::Arc, time::Instant};
use tcn::SignedReport;

#[derive(Copy, Clone)]
struct Element {}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Alert {
    pub id: String,
    pub report_id: String,

    pub report: PublicReport,

    // Note: for now these fields "raw", as this struct is used for FFI.
    // if it's needed to manipulate Alert in Rust, a separate type should be created.
    pub contact_start: u64,
    pub contact_end: u64,
    pub min_distance: f32,
    pub avg_distance: f32,
}

pub trait SignedReportExt {
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

pub struct ReportsUpdater<
    'a,
    T: Preferences,
    U: TcnDao,
    V: TcnMatcher,
    W: TcnApi,
    X: MemoMapper,
    Y: AlertDao,
> {
    pub preferences: Arc<T>,
    pub tcn_dao: Arc<U>,
    pub tcn_matcher: V,
    pub api: &'a W,
    pub memo_mapper: &'a X,
    pub exposure_grouper: ExposureGrouper,
    pub alert_dao: Arc<Y>,
}

impl<'a, T, U, V, W, X, Y> ReportsUpdater<'a, T, U, V, W, X, Y>
where
    T: Preferences,
    U: TcnDao,
    V: TcnMatcher,
    W: TcnApi,
    X: MemoMapper,
    Y: AlertDao,
{
    pub fn update_and_fetch_alerts(&self) -> Result<Vec<Alert>, ServicesError> {
        self.update_alerts()?;
        self.alert_dao.all()
    }

    fn update_alerts(&self) -> Result<(), ServicesError> {
        let new_alerts = self.fetch_new_reports()?;
        self.alert_dao.save(new_alerts)
    }

    fn fetch_new_reports(&self) -> Result<Vec<Alert>, ServicesError> {
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
            id: format!("{:?}", signed_report.sig), // TODO this is wrong now: one report can have multiple alerts
            report_id: format!("{:?}", signed_report.sig),
            report: public_report,
            contact_start: measurements.contact_start.value,
            contact_end: measurements.contact_end.value,
            min_distance: measurements.min_distance,
            avg_distance: measurements.avg_distance,
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

    fn store_last_completed_interval(&self, intervals: Vec<ReportsInterval>, now: &UnixTime) {
        let interval = ReportsInterval::interval_ending_before(intervals.clone(), now);
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
}
