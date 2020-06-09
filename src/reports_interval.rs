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
