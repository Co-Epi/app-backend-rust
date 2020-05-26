use crate::{networking::{TcnApi, NetworkingError}, reports_interval, DB_UNINIT, DB, byte_vec_to_16_byte_array, errors::{Error, ServicesError}, preferences::{PreferencesKey, Preferences}, byte_vec_to_24_byte_array, byte_vec_to_8_byte_array};
use reports_interval::{ReportsInterval, UnixTime};
use tcn::{TemporaryContactNumber, SignedReport};
use std::{time::Instant, sync::Arc};
use serde::Serialize;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

pub trait TcnMatcher {
  fn match_reports(&self, tcns: Vec<ObservedTcn>, reports: Vec<SignedReport>) -> Result<Vec<MatchedReport>, ServicesError>;
}

pub struct TcnMatcherImpl {}
impl TcnMatcher for TcnMatcherImpl {
  fn match_reports(&self, tcns: Vec<ObservedTcn>, reports: Vec<SignedReport>) -> Result<Vec<MatchedReport>, ServicesError> {
    Self::match_reports_with(tcns, reports)
  }
}
 
#[derive(Debug, Clone)]
pub struct MatchedReport{ report: SignedReport, contact_time: UnixTime }

// TODO remove duplicate matcher functions from lib.rs
impl TcnMatcherImpl {

  // TODO use TCN repo's match_btreeset test code? Compare performance.
  pub fn match_reports_with(tcns: Vec<ObservedTcn>, reports: Vec<SignedReport>) -> Result<Vec<MatchedReport>, ServicesError> {
    let mut out: Vec<MatchedReport> = Vec::new();

    let observed_tcns_map: HashMap<[u8; 16], ObservedTcn> = tcns.into_iter().map(|e|
      (e.tcn.0, e)
    ).collect();

    for report in reports {
      // TODO no unwrap
      let rep = report.clone().verify().unwrap();
      for tcn in rep.temporary_contact_numbers() {
        if let Some(entry) = observed_tcns_map.get(&tcn.0) {
          out.push(MatchedReport { report, contact_time: entry.time.clone() });
          break;
        }
      }
    }

    Ok(out)
  }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ObservedTcn { tcn: TemporaryContactNumber, time: UnixTime }

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
      time: UnixTime{ value: time }
    }
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
    let observed_tcn = ObservedTcn { 
      tcn: TemporaryContactNumber(byte_vec_to_16_byte_array(bytes_vec)), 
      time: UnixTime { value: Utc::now().timestamp() as u64 }
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
      .ok_or(DB_UNINIT).map_err(Error::from)?
      .scan("tcn").map_err(Error::from)?;

    for (_id,content) in items {
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
  memo: String,
  contact_time: u64,
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

      signed_reports.into_iter().filter_map(|matched_report| {
        match matched_report.report.verify() {
          Ok(report) => Some(Alert {
            // TODO(important) id should be derived from report.
            // TODO random UUIDs allow duplicate alerts in the DB, which is what we're trying to prevent.
            // TODO Maybe add id field in TCN library. Everything is currently private.
            id: format!("{:?}", Uuid::new_v4()),
            memo: format!("{:?}", report.memo_data()),
            contact_time: matched_report.contact_time.value
          }),
          Err(error) =>  {
            println!("Couldn't get report from signed, error: {:?}", error);
            None
          }
        }
      }).collect()
    )
  }

  fn retrieve_and_match_new_reports(&self) -> Result<Vec<MatchedReport>, ServicesError> {
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

  fn find_matches(&self, reports: Vec<SignedReport>) -> Result<Vec<MatchedReport>, ServicesError> {
    let matching_start_time = Instant::now();

    println!("R Start matching...");

    let tcns = self.tcn_dao.all();
    println!("R DB TCNs count: {:?}", tcns);

    let matched_reports: Result<Vec<MatchedReport>, ServicesError> = tcns
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
  matched: Vec<MatchedReport>,
  interval: ReportsInterval 
}

#[derive(Debug, Clone)]
struct SignedReportsChunk { 
  reports: Vec<SignedReport>,
  interval: ReportsInterval 
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tcn_saved_and_restored_from_bytes() {
    let mut tcn_bytes: [u8; 16] = [0; 16];
    tcn_bytes[1] = 1;
    tcn_bytes[12] = 5;
    tcn_bytes[15] = 250;
    let time = UnixTime { value: 1590528300 };
    let observed_tcn = ObservedTcn { tcn: TemporaryContactNumber(tcn_bytes), time };
    let observed_tcn_as_bytes = observed_tcn.as_bytes();
    let observed_tc_from_bytes = ObservedTcn::from_bytes(observed_tcn_as_bytes);
    assert_eq!(observed_tc_from_bytes, observed_tcn);
  }
}
