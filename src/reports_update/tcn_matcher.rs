use crate::{errors::ServicesError, tcn_recording::observed_tcn_processor::ObservedTcn};
use log::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tcn::SignedReport;

pub trait TcnMatcher {
    fn match_reports(
        &self,
        tcns: Vec<ObservedTcn>,
        reports: Vec<SignedReport>,
    ) -> Result<Vec<MatchedReport>, ServicesError>;
}

#[derive(Debug, Clone)]
pub struct MatchedReport {
    pub report: SignedReport,
    pub tcns: Vec<ObservedTcn>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        reporting::{
            memo::{MemoMapper, MemoMapperImpl},
            public_report::{CoughSeverity, FeverSeverity, PublicReport},
            symptom_inputs::UserInput,
        },
        reports_interval::UnixTime,
        reports_update::reports_updater::SignedReportExt,
        signed_report_to_bytes,
    };
    use std::time::Instant;
    use tcn::{MemoType, ReportAuthorizationKey, TemporaryContactNumber};

    #[test]
    fn one_report_matches() {
        let verification_report_str = "D7Z8XrufMgfsFH3K5COnv17IFG2ahDb4VM/UMK/5y0+/OtUVVTh7sN0DQ5+R+ocecTilR+SIIpPHzujeJdJzugEAECcAFAEAmmq5XgAAAACaarleAAAAACEBo8p1WdGeXb5O5/3kN6x7GSylgiYGIGsABl3NrxhJu9XHwsN3f6yvRwUxs2fhP4oU5E3+JWabBP6v09pGV1xRCw==";
        let verification_report_tcn: [u8; 16] = [
            24, 229, 125, 245, 98, 86, 219, 221, 172, 25, 232, 150, 206, 66, 164, 173,
        ]; // belongs to report
        let verification_contact_start = UnixTime { value: 1590528300 };
        let verification_contact_end = UnixTime { value: 1590528301 };
        let verification_min_distance = 2.3;
        let verification_avg_distance = 3.0;
        let verification_total_count = 3;
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber(verification_report_tcn),
                contact_start: verification_contact_start.clone(),
                contact_end: verification_contact_end.clone(),
                min_distance: verification_min_distance,
                avg_distance: verification_avg_distance,
                total_count: verification_total_count,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber(verification_report_tcn),
                contact_start: verification_contact_time.clone(),
                contact_end: verification_contact_time.clone(),
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([1; 16]),
                contact_start: UnixTime { value: 1590528300 },
                contact_end: UnixTime { value: 1590528301 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
