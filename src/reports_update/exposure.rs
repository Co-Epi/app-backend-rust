use crate::{
    errors::ServicesError, reports_interval, tcn_recording::observed_tcn_processor::ObservedTcn,
};
use reports_interval::UnixTime;

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

        let mut min_distance = std::f32::MAX;
        let mut total_count: usize = 0;
        let mut avg_distance = 0.0;
        for tcn in tcns {
            min_distance = f32::min(min_distance, tcn.min_distance);
            total_count += tcn.total_count;
            avg_distance += tcn.avg_distance * tcn.total_count as f32;
        }
        // Note: this struct (Exposure) guarantees that TCNs can't be empty,
        // so don't have to check for 0 division.
        avg_distance /= total_count as f32;

        ExposureMeasurements {
            contact_start: UnixTime {
                value: contact_start,
            },
            contact_end: UnixTime { value: contact_end },
            min_distance,
            avg_distance,
            total_count,
        }
    }
}
pub struct ExposureMeasurements {
    pub contact_start: UnixTime,
    pub contact_end: UnixTime,
    pub min_distance: f32,
    pub avg_distance: f32,
    pub total_count: usize,
}

// Groups TCNs by contiguity.
#[derive(Clone)]
pub struct ExposureGrouper {
    pub threshold: u64,
}

impl ExposureGrouper {
    pub fn group(&self, mut tcns: Vec<ObservedTcn>) -> Vec<Exposure> {
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
    pub fn is_contiguous(&self, tcn1: &ObservedTcn, tcn2: &ObservedTcn) -> bool {
        // Signed: overlap (start2 < end1) considered contiguous.
        (tcn2.contact_start.value as i64 - tcn1.contact_end.value as i64) < self.threshold as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tcn::TemporaryContactNumber;

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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1500 },
                contact_end: UnixTime { value: 1501 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 2002 },
                contact_end: UnixTime { value: 2501 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 1001 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            // starts before previous TCN ends
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1600 },
                contact_end: UnixTime { value: 2600 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1 },
                contact_end: UnixTime { value: 2 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 3900 },
                contact_end: UnixTime { value: 4500 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 500 },
                contact_end: UnixTime { value: 501 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1589209754 },
                contact_end: UnixTime { value: 1589209755 },
                min_distance: 0.0,
                avg_distance: 0.0,
                total_count: 1,
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
                avg_distance: 2.7, // (2.3 + 3.1) / 2
                total_count: 2,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 2601 },
                contact_end: UnixTime { value: 3223 },
                min_distance: 0.845,
                avg_distance: 0.948333333, // (0.845 + 0.5 + 1.5) / 3
                total_count: 3,
            },
            ObservedTcn {
                tcn: TemporaryContactNumber([0; 16]),
                contact_start: UnixTime { value: 1000 },
                contact_end: UnixTime { value: 2000 },
                min_distance: 0.846,
                avg_distance: 0.846,
                total_count: 1,
            },
        ];

        let measurements = Exposure::create_with_tcns(tcns).unwrap().measurements();

        assert_eq!(measurements.contact_start.value, 1000);
        assert_eq!(measurements.contact_end.value, 3223);
        assert_eq!(measurements.min_distance, 0.845);
        let avg_rounded = (measurements.avg_distance * 10000.0).floor() / 10000.0;
        assert_eq!(avg_rounded, 1.5151); // (2.3 + 3.1 + 0.845 + 0.5 + 1.5 + 0.846) / (2 + 3 + 1)
        assert_eq!(measurements.total_count, 6); // 2 + 3 + 1
    }
}
