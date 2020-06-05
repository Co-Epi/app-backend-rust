use super::{
    bit_vector::BitVector,
    mappers::{
        BitMapper, BitVectorMappable, BoolMapper, CoughSeverityMapper, FeverSeverityMapper,
        TimeMapper, TimeUserInputMapper, VersionMapper,
    },
    public_report::PublicReport,
};
use crate::reports_interval::UnixTime;
use std::convert::TryInto;

pub struct Memo {
    pub bytes: Vec<u8>,
}
pub trait MemoMapper {
    fn to_memo(&self, report: PublicReport, time: UnixTime) -> Memo;
    fn to_report(&self, memo: Memo) -> PublicReport;
}

pub struct MemoMapperImpl {}

impl MemoMapperImpl {
    const VERSION_MAPPER: VersionMapper = VersionMapper {};
    const TIME_MAPPER: TimeMapper = TimeMapper {};
    const TIME_USER_INPUT_MAPPER: TimeUserInputMapper = TimeUserInputMapper {};
    const COUGH_SEVERITY_MAPPER: CoughSeverityMapper = CoughSeverityMapper {};
    const FEVER_SEVERITY_MAPPER: FeverSeverityMapper = FeverSeverityMapper {};
    const BOOLEAN_MAPPER: BoolMapper = BoolMapper {};
}

impl MemoMapper for MemoMapperImpl {
    fn to_memo(&self, report: PublicReport, time: UnixTime) -> Memo {
        let memo_version: u16 = 1;

        let bits = vec![
            Self::VERSION_MAPPER.to_bits(memo_version),
            Self::TIME_MAPPER.to_bits(time),
            Self::TIME_USER_INPUT_MAPPER.to_bits(report.earliest_symptom_time),
            Self::COUGH_SEVERITY_MAPPER.to_bits(report.cough_severity),
            Self::FEVER_SEVERITY_MAPPER.to_bits(report.fever_severity),
            Self::BOOLEAN_MAPPER.to_bits(report.breathlessness),
        ];

        Memo {
            bytes: bits
                .into_iter()
                .fold(BitVector { bits: vec![] }, |acc, e| acc.concat(e))
                .as_u8_array(),
        }
    }

    fn to_report(&self, memo: Memo) -> PublicReport {
        let bits: Vec<bool> = memo
            .bytes
            .into_iter()
            .flat_map(|byte| byte.to_bits().bits)
            .collect();

        let mut next: usize = 0;

        // Version for now not handled
        let _ = extract(&bits, &Self::VERSION_MAPPER, next).value(|v| next += v);

        // TODO handle report time?
        let _ = extract(&bits, &Self::TIME_MAPPER, next).value(|v| next += v);

        let earliest_symptom_time =
            extract(&bits, &Self::TIME_USER_INPUT_MAPPER, next).value(|v| next += v);
        let cough_severity =
            extract(&bits, &Self::COUGH_SEVERITY_MAPPER, next).value(|v| next += v);
        let fever_severity =
            extract(&bits, &Self::FEVER_SEVERITY_MAPPER, next).value(|v| next += v);
        let breathlessness = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);

        PublicReport {
            earliest_symptom_time,
            fever_severity,
            cough_severity,
            breathlessness,
        }
    }
}

struct ExtractResult<T> {
    value: T,
    count: usize,
}

impl<T> ExtractResult<T> {
    // Convenience to parse memo with less boilerplate
    fn value<F: FnOnce(usize) -> ()>(self, f: F) -> T {
        f(self.count);
        self.value
    }
}

fn extract<T>(bits: &Vec<bool>, mapper: &dyn BitMapper<T>, start: usize) -> ExtractResult<T> {
    let end = mapper.bit_count() + start;
    let sub_bits: Vec<bool> = bits[start..end]
        .try_into()
        .expect("Couldn't convert bits into vector");

    ExtractResult {
        value: mapper.from_bits(BitVector { bits: sub_bits }),
        count: mapper.bit_count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::public_report::{CoughSeverity, FeverSeverity};
    use crate::reporting::symptom_inputs::UserInput;

    #[test]
    fn maps_no_symptoms() {
        let memo_mapper = MemoMapperImpl {};

        let report = PublicReport {
            earliest_symptom_time: UserInput::None,
            fever_severity: FeverSeverity::None,
            breathlessness: false,
            cough_severity: CoughSeverity::None,
        };

        let memo: Memo = memo_mapper.to_memo(report.clone(), UnixTime { value: 1589209754 });
        let mapped_report: PublicReport = memo_mapper.to_report(memo);

        assert_eq!(mapped_report, report.clone());
    }

    #[test]
    fn maps_all_symptoms_set_arbitrary() {
        let memo_mapper = MemoMapperImpl {};

        let report = PublicReport {
            earliest_symptom_time: UserInput::Some(UnixTime { value: 1589209754 }),
            fever_severity: FeverSeverity::Serious,
            breathlessness: true,
            cough_severity: CoughSeverity::Existing,
        };

        let memo: Memo = memo_mapper.to_memo(report.clone(), UnixTime { value: 0 });
        let mapped_report: PublicReport = memo_mapper.to_report(memo);

        assert_eq!(mapped_report, report.clone());
    }
}
