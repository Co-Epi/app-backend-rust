use super::{
    bit_vector::BitVector,
    public_report::{CoughSeverity, FeverSeverity},
    symptom_inputs::UserInput,
};
use crate::{expect_log, reports_interval::UnixTime};
#[cfg(target_os = "android")]
use log::error;

pub trait BitMapper<T> {
    fn bit_count(&self) -> usize;

    fn to_bits(&self, value: T) -> BitVector {
        let bits = self.to_bits_unchecked(value);
        if bits.len() != self.bit_count() {
            panic!(
                "Incorrect bit count: {}. Required: {}",
                bits.len(),
                self.bit_count()
            )
        } else {
            bits
        }
    }

    fn from_bits(&self, bit_vector: BitVector) -> T {
        if bit_vector.len() != self.bit_count() {
            panic!(
                "Incorrect bit count: {}. Required: {}",
                bit_vector.len(),
                self.bit_count()
            )
        }
        self.from_bits_unchecked(bit_vector)
    }

    fn to_bits_unchecked(&self, value: T) -> BitVector;

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> T;
}

pub struct VersionMapper {}
impl BitMapper<u16> for VersionMapper {
    fn bit_count(&self) -> usize {
        16
    }

    fn to_bits_unchecked(&self, value: u16) -> BitVector {
        value.to_bits()
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> u16 {
        bit_vector.as_u16()
    }
}

pub struct TimeMapper {}
impl BitMapper<UnixTime> for TimeMapper {
    fn bit_count(&self) -> usize {
        64
    }

    fn to_bits_unchecked(&self, value: UnixTime) -> BitVector {
        value.value.to_bits()
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> UnixTime {
        UnixTime {
            value: bit_vector.as_u64(),
        }
    }
}

pub struct TimeUserInputMapper {}
impl BitMapper<UserInput<UnixTime>> for TimeUserInputMapper {
    fn bit_count(&self) -> usize {
        64
    }

    fn to_bits_unchecked(&self, value: UserInput<UnixTime>) -> BitVector {
        match value {
            UserInput::None => u64::MAX,
            UserInput::Some(input) => input.value,
        }
        .to_bits()
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> UserInput<UnixTime> {
        let value = bit_vector.as_u64();
        match value {
            u64::MAX => UserInput::None,
            _ => UserInput::Some(UnixTime { value }),
        }
    }
}

pub struct BoolMapper {}
impl BitMapper<bool> for BoolMapper {
    fn bit_count(&self) -> usize {
        1
    }

    fn to_bits_unchecked(&self, value: bool) -> BitVector {
        BitVector { bits: vec![value] }
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> bool {
        // unwrap: we know that _currently_ bit_vector can't be empty (because of the payload and memo's max length)
        // TODO safer. Ideally without having to change interface to return Result.
        bit_vector.bits.first().unwrap().to_owned()
    }
}

pub struct CoughSeverityMapper {}
impl BitMapper<CoughSeverity> for CoughSeverityMapper {
    fn bit_count(&self) -> usize {
        4
    }

    fn to_bits_unchecked(&self, value: CoughSeverity) -> BitVector {
        value.raw_value().to_bits().as_unibble_bit_vector()
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> CoughSeverity {
        let value = bit_vector.as_u8();
        let res = CoughSeverity::from(value);
        expect_log!(res, "Not supported raw value")
    }
}

pub struct FeverSeverityMapper {}
impl BitMapper<FeverSeverity> for FeverSeverityMapper {
    fn bit_count(&self) -> usize {
        4
    }

    fn to_bits_unchecked(&self, value: FeverSeverity) -> BitVector {
        value.raw_value().to_bits().as_unibble_bit_vector()
    }

    fn from_bits_unchecked(&self, bit_vector: BitVector) -> FeverSeverity {
        let value = bit_vector.as_u8();
        let res = FeverSeverity::from(value);
        expect_log!(res, "Not supported raw value")
    }
}

pub trait BitVectorMappable {
    fn to_bits(&self) -> BitVector;
}

impl BitVectorMappable for u64 {
    fn to_bits(&self) -> BitVector {
        let bits: Vec<bool> = (0..64)
            .map(|index| {
                let value: Self = (self >> index) & 0x01;
                value == 1
            })
            .collect();
        BitVector { bits }
    }
}

impl BitVectorMappable for u16 {
    fn to_bits(&self) -> BitVector {
        let bits: Vec<bool> = (0..16)
            .map(|index| {
                let value: Self = (self >> index) & 0x01;
                value == 1
            })
            .collect();
        BitVector { bits }
    }
}

impl BitVectorMappable for u8 {
    fn to_bits(&self) -> BitVector {
        let bits: Vec<bool> = (0..8)
            .map(|index| {
                let value: Self = (self >> index) & 0x01;
                value == 1
            })
            .collect();
        BitVector { bits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_logger;
    use log::*;

    #[test]
    fn version_mapper_maps_10_to_bits() {
        simple_logger::setup();
        let version_mapper = VersionMapper {};
        let bit_vector = version_mapper.to_bits(10);

        let mut bits = vec![false; 16];
        bits[1] = true;
        bits[3] = true;

        debug!("Mapper bits : {:?}", bits);

        assert_eq!(bits, bit_vector.bits);
    }

    #[test]
    fn version_mapper_maps_bits_to_10() {
        let version_mapper = VersionMapper {};

        let mut bits = vec![false; 16];
        bits[1] = true;
        bits[3] = true;

        let number = version_mapper.from_bits(BitVector { bits });
        debug!("Number : {:?}", number);
        assert_eq!(number, 10);
    }
}
