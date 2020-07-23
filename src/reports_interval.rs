use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct UnixTime {
    pub value: u64,
}

impl UnixTime {
    pub fn now() -> UnixTime {
        UnixTime {
            value: Utc::now().timestamp() as u64,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ReportsInterval {
    pub number: u64,
    pub length: u64,
}

impl ReportsInterval {
    pub fn next(&self) -> ReportsInterval {
        ReportsInterval {
            number: self.number + 1,
            length: self.length,
        }
    }

    pub fn start(&self) -> u64 {
        self.number * self.length
    }

    pub fn end(&self) -> u64 {
        self.start() + self.length
    }

    pub fn starts_before(&self, time: &UnixTime) -> bool {
        self.start() < time.value
    }

    pub fn ends_before(&self, time: &UnixTime) -> bool {
        self.end() < time.value
    }

    pub fn create_for_with_default_length(time: &UnixTime) -> ReportsInterval {
        Self::create_for(time, 21600)
    }

    pub fn create_for(time: &UnixTime, length_seconds: u64) -> ReportsInterval {
        ReportsInterval {
            number: time.value / length_seconds,
            length: length_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        database::{preferences::PreferencesImpl, tcn_dao::TcnDaoImpl},
        networking::TcnApiImpl,
        reporting::memo::MemoMapperImpl,
        reports_update::{reports_updater::ReportsUpdater, tcn_matcher::TcnMatcherRayon},
    };

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

        // TODO move this function outside of ReportsUpdater
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

        // TODO move this function outside of ReportsUpdater
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
}
