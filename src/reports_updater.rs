use crate::{networking::{TcnApi, NetworkingError}, reports_interval, all_stored_tcns, Res};
use reports_interval::{ ReportsInterval, UnixTime };
use tcn::{TemporaryContactNumber, SignedReport};
use std::{collections::HashSet, time::Instant};

trait TcnMatcher {
  fn match_tcns(&self, tcns: &Vec<TemporaryContactNumber>, reports: &Vec<SignedReport>) -> Vec<SignedReport>;
}

struct TcnMatcherImpl {}
impl TcnMatcher for TcnMatcherImpl {
  fn match_tcns(&self, tcns: &Vec<TemporaryContactNumber>, reports: &Vec<SignedReport>) -> Vec<SignedReport> {
    // TODO no unwrap
    Self::match_reports(reports).unwrap()
  }
}

// TODO remove duplicate matcher functions from lib.rs
impl TcnMatcherImpl {
  // TODO use TCN repo's match_btreeset test code? Compare performance.
  fn match_reports<'a, I: Iterator<Item = &'a SignedReport>>(reports: I) -> Res<Vec<&'a SignedReport>> {
    let stored_tcns: HashSet<u128> = all_stored_tcns()?.into_iter().collect();
    Self::match_reports_with(stored_tcns, reports)
  }

  fn match_reports_with<'a, I: Iterator<Item = &'a SignedReport>>(tcns: HashSet<u128>, reports: I) -> Res<Vec<&'a SignedReport>> {
    let mut out: Vec<&SignedReport> = Vec::new();
    for report in reports {
      // TODO no unwrap
      let rep = report.verify().unwrap();
      for tcn in rep.temporary_contact_numbers() {
        if tcns.contains(&u128::from_le_bytes(tcn.0)) {
          out.push(report);
          break;
        }
      }
    }

    Ok(out)
  }
}

trait TcnDao {
  fn all(&self) -> Vec<TemporaryContactNumber>;
}

struct TcnDaoImpl {}
impl TcnDao for TcnDaoImpl {
  fn all(&self) -> Vec<TemporaryContactNumber> {
    // TODO integrate related functions from lib.rs 
    unimplemented!();
  }
}

enum PreferencesKey {
  SeenOnboarding,
  LastCompletedReportsInterval
}

trait Preferences {
  fn get_object<T>(&self, key: PreferencesKey) -> Option<T>;
  fn put_object<T>(&self, key: PreferencesKey, value: T);
}

struct PreferencesImpl {}
impl Preferences for PreferencesImpl {

  fn get_object<T>(&self, key: PreferencesKey) -> Option<T> {
    // TODO key value store (prob third party? And manage object as JSON, probably)
    unimplemented!();
  }

  fn put_object<T>(&self, key: PreferencesKey, value: T) {
    // TODO key value store (prob third party? And manage object as JSON, probably)
    unimplemented!();
  }
}

struct ReportsUpdater<
  PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi
> {
  preferences: PreferencesType, tcn_dao: TcnDaoType, tcn_matcher: TcnMatcherType, api: ApiType
}

impl<
  PreferencesType: Preferences, TcnDaoType: TcnDao, TcnMatcherType: TcnMatcher, ApiType: TcnApi
> ReportsUpdater<PreferencesType, TcnDaoType, TcnMatcherType, ApiType> {

  fn retrieve_and_match_new_reports(&self) -> Result<Vec<SignedReport>, NetworkingError> {
    let now: UnixTime = UnixTime::now();
    self.matching_reports(
      self.determine_start_interval(&now),
      &now
    )
    .do_if_ok(|matched_reports| {
      let intervals = matched_reports.into_iter().map(|c| c.interval).collect();
      self.store_last_completed_interval(intervals, &now);
    })
    .map (|chunks| 
      chunks.into_iter().flat_map(|chunk| chunk.matched).collect()
    )
  }

  fn retrieve_last_completed_interval(&self) -> Option<ReportsInterval> {
    self.preferences.get_object(PreferencesKey::LastCompletedReportsInterval)
  }

  fn determine_start_interval(&self, time: &UnixTime) -> ReportsInterval {
    self.retrieve_last_completed_interval()
      .map(|interval| interval.next())
      .unwrap_or(ReportsInterval::create_for_with_default_length(time))
  }

  fn matching_reports(&self, startInterval: ReportsInterval, until: &UnixTime) -> Result<Vec<MatchedReportsChunk>, NetworkingError> {
    let sequence = Self::generate_intervals_sequence(startInterval, until);
    let reports = sequence.map (|interval| self.retrieve_reports(interval));
    let matched_result = reports
      .map (|interval| self.match_retrieved_reports_result(interval));
    matched_result.collect()
  }

  fn generate_intervals_sequence(mut from: ReportsInterval, until: &UnixTime) -> impl Iterator<Item = ReportsInterval> + '_ {
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

  fn match_retrieved_reports_result(&self, reports_result: Result<SignedReportsChunk, NetworkingError>) -> Result<MatchedReportsChunk, NetworkingError> {
    reports_result.map (|chunk| self.to_matched_reports_chunk(&chunk))
  }

  /**
  * Maps reports chunk to a new chunk containing possible matches.
  */
  fn to_matched_reports_chunk(&self, chunk: &SignedReportsChunk) -> MatchedReportsChunk {
    MatchedReportsChunk { 
      reports: chunk.reports.clone(), 
      matched: self.find_matches(&chunk.reports), 
      interval: chunk.interval.clone()
    }
  }

  fn find_matches(&self, reports: &Vec<SignedReport>) -> Vec<SignedReport> {
    let matching_start_time = Instant::now();

    println!("R Start matching...");

    let tcns = self.tcn_dao.all();
    println!("R DB TCNs count: {:?}", tcns);

    let matched_reports: Vec<SignedReport> = self.tcn_matcher.match_tcns(&tcns, reports);

    let time = matching_start_time.elapsed().as_secs();
    println!("Took ${:?}s to match reports", time);

    if !matched_reports.is_empty() {
        println!("Matches found ({:?}): {:?}", matched_reports.len(), matched_reports);
    } else {
        println!("No matches found");
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
      self.preferences.put_object(PreferencesKey::LastCompletedReportsInterval, interval);
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

trait ResultExtensions {
  type Ok;
  type Error;

  fn do_if_ok<F: FnOnce(&Self::Ok) -> ()>(self, f: F) -> Self;
}

impl <T, E> ResultExtensions for Result<T, E> {
  type Ok = T;
  type Error = E;

  fn do_if_ok<F: FnOnce(&T) -> ()>(self, f: F) -> Self {
    match &self {
      Ok(value) => f(&value),
      Err(_) => {}
    }
    self
  }
}

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
