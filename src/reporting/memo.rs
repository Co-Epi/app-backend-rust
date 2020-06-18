use super::{
    bit_vector::BitVector,
    mappers::{
        BitMapper, BitVectorMappable, BoolMapper, CoughSeverityMapper, FeverSeverityMapper,
        TimeMapper, TimeUserInputMapper, VersionMapper,
    },
    public_report::PublicReport,
};
use std::convert::TryInto;

pub struct Memo {
    pub bytes: Vec<u8>,
}
pub trait MemoMapper {
    fn to_memo(&self, report: PublicReport) -> Memo;
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
    fn to_memo(&self, report: PublicReport) -> Memo {
        let memo_version: u16 = 1;

        let bits = vec![
            Self::VERSION_MAPPER.to_bits(memo_version),
            Self::TIME_MAPPER.to_bits(report.report_time),
            Self::TIME_USER_INPUT_MAPPER.to_bits(report.earliest_symptom_time),
            Self::COUGH_SEVERITY_MAPPER.to_bits(report.cough_severity),
            Self::FEVER_SEVERITY_MAPPER.to_bits(report.fever_severity),
            Self::BOOLEAN_MAPPER.to_bits(report.breathlessness),
            Self::BOOLEAN_MAPPER.to_bits(report.muscle_aches),
            Self::BOOLEAN_MAPPER.to_bits(report.loss_smell_or_taste),
            Self::BOOLEAN_MAPPER.to_bits(report.diarrhea),
            Self::BOOLEAN_MAPPER.to_bits(report.runny_nose),
            Self::BOOLEAN_MAPPER.to_bits(report.other),
            Self::BOOLEAN_MAPPER.to_bits(report.no_symptoms),
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

        let report_time = extract(&bits, &Self::TIME_MAPPER, next).value(|v| next += v);

        let earliest_symptom_time =
            extract(&bits, &Self::TIME_USER_INPUT_MAPPER, next).value(|v| next += v);
        let cough_severity =
            extract(&bits, &Self::COUGH_SEVERITY_MAPPER, next).value(|v| next += v);
        let fever_severity =
            extract(&bits, &Self::FEVER_SEVERITY_MAPPER, next).value(|v| next += v);
        let breathlessness = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let muscle_aches = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let loss_smell_or_taste = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let diarrhea = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let runny_nose = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let other = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);
        let no_symptoms = extract(&bits, &Self::BOOLEAN_MAPPER, next).value(|v| next += v);

        PublicReport {
            report_time,
            earliest_symptom_time,
            fever_severity,
            cough_severity,
            breathlessness,
            muscle_aches,
            loss_smell_or_taste,
            diarrhea,
            runny_nose,
            other,
            no_symptoms,
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
    use crate::reports_interval::UnixTime;

    #[test]
    fn maps_nothing_set() {
        let memo_mapper = MemoMapperImpl {};

        let report = PublicReport {
            report_time: UnixTime { value: 1589209754 },
            earliest_symptom_time: UserInput::None,
            fever_severity: FeverSeverity::None,
            cough_severity: CoughSeverity::None,
            breathlessness: false,
            muscle_aches: false,
            loss_smell_or_taste: false,
            diarrhea: false,
            runny_nose: false,
            other: false,
            no_symptoms: false,
        };

        let memo: Memo = memo_mapper.to_memo(report.clone());
        let mapped_report: PublicReport = memo_mapper.to_report(memo);

        assert_eq!(mapped_report, report.clone());
    }

    #[test]
    fn maps_all_symptoms_set_arbitrary() {
        let memo_mapper = MemoMapperImpl {};

        let report = PublicReport {
            report_time: UnixTime { value: 0 },
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

        let memo: Memo = memo_mapper.to_memo(report.clone());
        let mapped_report: PublicReport = memo_mapper.to_report(memo);

        assert_eq!(mapped_report, report.clone());
    }
}
