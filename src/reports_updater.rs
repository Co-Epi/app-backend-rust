use crate::{networking::{TcnApi, NetworkingError}, reports_interval, DB_UNINIT, DB, byte_vec_to_16_byte_array, errors::{Error, ServicesError}, preferences::{PreferencesKey, Preferences}};
use reports_interval::{ReportsInterval, UnixTime};
use tcn::{TemporaryContactNumber, SignedReport};
use std::{collections::HashSet, time::Instant, sync::Arc};
use serde::Serialize;
use uuid::Uuid;

pub trait TcnMatcher {
  fn match_reports(&self, tcns: Vec<u128>, reports: Vec<SignedReport>) -> Result<Vec<SignedReport>, ServicesError>;
}

pub struct TcnMatcherImpl {}
impl TcnMatcher for TcnMatcherImpl {
  fn match_reports(&self, tcns: Vec<u128>, reports: Vec<SignedReport>) -> Result<Vec<SignedReport>, ServicesError> {
    Self::match_reports_with(tcns, reports)
  }
}

// TODO remove duplicate matcher functions from lib.rs
impl TcnMatcherImpl {

  // TODO use TCN repo's match_btreeset test code? Compare performance.
  pub fn match_reports_with(tcns: Vec<u128>, reports: Vec<SignedReport>) -> Result<Vec<SignedReport>, ServicesError> {
    let mut out: Vec<SignedReport> = Vec::new();
    let tcns_set: HashSet<u128> = tcns.into_iter().collect();
    for report in reports {
      // TODO no unwrap
      let rep = report.clone().verify().unwrap();
      for tcn in rep.temporary_contact_numbers() {
        if tcns_set.contains(&u128::from_le_bytes(tcn.0)) {
          out.push(report);
          break;
        }
      }
    }

    Ok(out)
  }
}

pub trait ObservedTcnProcessor {
  fn save(&self, tcn_str: &str)  -> Result<(), ServicesError>;
}

pub struct ObservedTcnProcessorImpl<'a, A: TcnDao> {
  pub tcn_dao: &'a A
}

impl <'a, A: TcnDao> ObservedTcnProcessor for ObservedTcnProcessorImpl<'a, A> {
  fn save(&self, tcn_str: &str) -> Result<(), ServicesError> {
    let bytes_vec: Vec<u8> = hex::decode(tcn_str)?;
    let tcn: TemporaryContactNumber = TemporaryContactNumber(byte_vec_to_16_byte_array(bytes_vec));
    self.tcn_dao.save(tcn)
  }
}

pub trait TcnDao {
  fn all(&self) -> Result<Vec<u128>, ServicesError>;
  fn save(&self, tcn: TemporaryContactNumber) -> Result<(), ServicesError>;
}

pub struct TcnDaoImpl {}
impl TcnDao for TcnDaoImpl {
  fn all(&self) -> Result<Vec<u128>, ServicesError> {
    let mut out: Vec<u128> = Vec::new();

    let items = DB
        .get()
        .ok_or(DB_UNINIT).map_err(Error::from)?
        .scan("tcn").map_err(Error::from)?;

    for (_id,content) in items {
      let byte_array: [u8; 16] = byte_vec_to_16_byte_array(content);
      let tcn_bits: u128 = u128::from_le_bytes(byte_array);
      out.push(tcn_bits);
    }
    Ok(out)
  }

  fn save(&self, tcn: TemporaryContactNumber) -> Result<(), ServicesError> {
    let db = DB.get().ok_or(DB_UNINIT)?;
    let mut tx = db.begin()?;
    tx.insert_record("tcn", &tcn.0)?; // [u8; 16]
    // tx.put(CENS_BY_TS, ts, u128_of_tcn(tcn))?;
    tx.prepare_commit()?.commit()?;
    Ok(())
  }
}


#[derive(Debug, Serialize)]
pub struct Alert {
  id: String,
  memo: String,
  // TODO date: Note: Contact date (port from Android)
}

pub struct ReportsUpdater<'a, 
  PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi
> {
  pub preferences: Arc<PreferencesType>, 
  pub tcn_dao: &'a TcnDaoType, 
  pub tcn_matcher: TcnMatcherType, 
  pub api: &'a ApiType
}

impl<'a, 
  PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi
> ReportsUpdater<'a, PreferencesType, TcnDaoType, TcnMatcherType, ApiType> {

  pub fn fetch_new_reports(&self) -> Result<Vec<Alert>, ServicesError> {
    self.retrieve_and_match_new_reports().map(|signed_reports|

      signed_reports.into_iter().filter_map(|signed_report| {
        match signed_report.verify() {
          Ok(report) => Some(Alert {
            // TODO(important) id should be derived from report.
            // TODO random UUIDs allow duplicate alerts in the DB, which is what we're trying to prevent.
            // TODO Maybe add id field in TCN library. Everything is currently private.
            id: format!("{:?}", Uuid::new_v4()),
            memo: format!("{:?}", report.memo_data()),
          }),
          Err(error) =>  {
            println!("Couldn't get report from signed, error: {:?}", error);
            None
          }
        }
      }).collect()
    )
  }

  fn retrieve_and_match_new_reports(&self) -> Result<Vec<SignedReport>, ServicesError> {
    let now: UnixTime = UnixTime::now();

    let matching_reports = self.matching_reports(
      self.determine_start_interval(&now),
      &now
    );

    if let Ok(matching_reports) = &matching_reports {
      let intervals = matching_reports.iter().map(|c| c.interval).collect();
      self.store_last_completed_interval(intervals, &now);
    };

    matching_reports.map (|chunks| 
      chunks.into_iter().flat_map(|chunk| chunk.matched).collect()
    ).map_err(ServicesError::from)
  }

  fn retrieve_last_completed_interval(&self) -> Option<ReportsInterval> {
    self.preferences.last_completed_reports_interval(PreferencesKey::LastCompletedReportsInterval)
  }

  fn determine_start_interval(&self, time: &UnixTime) -> ReportsInterval {
    self.retrieve_last_completed_interval()
      .map(|interval| interval.next())
      .unwrap_or(ReportsInterval::create_for_with_default_length(time))
  }

  fn matching_reports(&self, startInterval: ReportsInterval, until: &UnixTime) -> Result<Vec<MatchedReportsChunk>, ServicesError> {
    let sequence = Self::generate_intervals_sequence(startInterval, until);
    let reports = sequence.map (|interval| self.retrieve_reports(interval));
    let matched_results = reports
      .map (|interval| self.match_retrieved_reports_result(interval));
    matched_results
      .into_iter()
      .collect::<Result<Vec<MatchedReportsChunk>, ServicesError>>()
      .map_err(ServicesError::from)
  }

  fn generate_intervals_sequence(from: ReportsInterval, until: &UnixTime) -> impl Iterator<Item = ReportsInterval> + '_ {
    std::iter::successors(Some(from), |item| Some(item.next()))
    .take_while(move |item| item.starts_before(until))
  }

  fn retrieve_reports(&self, interval: ReportsInterval) -> Result<SignedReportsChunk, NetworkingError> {
    let reports_strings_result: Result<Vec<String>, NetworkingError> = 
      self.api.get_reports(interval.number, interval.length);

    reports_strings_result.map ( |report_strings|
        SignedReportsChunk {
            reports: report_strings
              .into_iter()
              .filter_map(|report_string|
                Self::to_signed_report(&report_string).also(|res|
                    if res.is_none() {
                        println!("Failed to convert report string: $it to report");
                    }
                )
              )
              .collect(),
            interval: interval
        }
    )
  }

  fn match_retrieved_reports_result(&self, reports_result: Result<SignedReportsChunk, NetworkingError>) -> Result<MatchedReportsChunk, ServicesError> {
    reports_result
      .map_err(ServicesError::from)
      .and_then (|chunk| self.to_matched_reports_chunk(&chunk))
  }

  /**
  * Maps reports chunk to a new chunk containing possible matches.
  */
  fn to_matched_reports_chunk(&self, chunk: &SignedReportsChunk) -> Result<MatchedReportsChunk, ServicesError> {
    self.find_matches(chunk.reports.clone()).map(|matches|
      MatchedReportsChunk {
        reports: chunk.reports.clone(), 
        matched: matches, 
        interval: chunk.interval.clone()
      }
    )
    .map_err(ServicesError::from)
  }

  fn find_matches(&self, reports: Vec<SignedReport>) -> Result<Vec<SignedReport>, ServicesError> {
    let matching_start_time = Instant::now();

    println!("R Start matching...");

    let tcns = self.tcn_dao.all();
    println!("R DB TCNs count: {:?}", tcns);

    let matched_reports: Result<Vec<SignedReport>, ServicesError> = tcns
      .and_then(|tcns| self.tcn_matcher.match_reports(tcns, reports));

    let time = matching_start_time.elapsed().as_secs();
    println!("Took {:?}s to match reports", time);

    if let Ok(reports) = &matched_reports {
      if !reports.is_empty() {
        println!("Matches found ({:?}): {:?}", reports.len(), matched_reports);
      } else {
        println!("No matches found");
      }
    };

    if let Err(error) = &matched_reports {
      println!("Matching error: ({:?})", error)
    }

    matched_reports
  }

  fn to_signed_report(report_string: &str) -> Option<SignedReport> {
    base64::decode(report_string)
    .map(|bytes| SignedReport::read( bytes.as_slice()).unwrap())
    .also(|res| if res.is_err() { print!("error!"); })
    .map_err(|err| {
      println!("Error: {}", err);
      err
    })
    .ok()
  }

  fn interval_ending_before(mut intervals: Vec<ReportsInterval>, time: &UnixTime) -> Option<ReportsInterval> {
    // TODO shorter version of this?
    let reversed: Vec<ReportsInterval> = intervals.into_iter().rev().collect();
    reversed.into_iter().find(|i| i.ends_before(&time))
  }

  fn store_last_completed_interval(&self, intervals: Vec<ReportsInterval>, now: &UnixTime) {
    let interval = Self::interval_ending_before(intervals, now);

    if let Some(interval) = interval {
      self.preferences.set_last_completed_reports_interval(PreferencesKey::LastCompletedReportsInterval, interval);
    }
  }
}

// To insert easily side effects in flows anywhere (from Kotlin)
trait Also: Sized {
  fn also<F: FnOnce(&Self) -> ()>(self, f: F) -> Self {
    f(&self);
    self
  }
}

impl<T> Also for T {}

#[derive(Debug, Clone)]
struct MatchedReportsChunk { 
  reports: Vec<SignedReport>,
  matched: Vec<SignedReport>,
  interval: ReportsInterval 
}

#[derive(Debug, Clone)]
struct SignedReportsChunk { 
  reports: Vec<SignedReport>,
  interval: ReportsInterval 
}
