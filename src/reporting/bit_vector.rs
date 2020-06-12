use std::{convert::TryInto, u64};

pub struct BitVector {
    pub bits: Vec<bool>,
}

impl BitVector {
    pub fn concat(&self, bit_vector: BitVector) -> BitVector {
        BitVector {
            bits: self
                .bits
                .clone()
                .into_iter()
                .chain(bit_vector.bits.into_iter())
                .collect(),
        }
    }

    pub fn as_u8_array(&self) -> Vec<u8> {
        let chunks: Vec<[bool; 8]> = Self::as_byte_chunks(self.bits.clone());
        chunks
            .into_iter()
            .map(|chunk| {
                let bit_string = Self::to_bit_string(chunk);
                let reversed: String = bit_string.chars().rev().collect(); // Most significant bit first
                u8::from_str_radix(reversed.as_ref(), 2).unwrap() // unwrap: we know that the chunks have 8 bits (are not empty) and composed of 1 and 0.
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn as_u8(&self) -> u8 {
        let arr: [u8; 1] = self
            .as_u8_array()
            .as_slice()
            .try_into()
            .expect("Unexpected size");
        unsafe { std::mem::transmute::<[u8; 1], u8>(arr) }
    }

    pub fn as_u16(&self) -> u16 {
        let arr: [u8; 2] = self
            .as_u8_array()
            .as_slice()
            .try_into()
            .expect("Unexpected size");
        unsafe { std::mem::transmute::<[u8; 2], u16>(arr) }
    }

    pub fn as_u64(&self) -> u64 {
        let arr: [u8; 8] = self
            .as_u8_array()
            .as_slice()
            .try_into()
            .expect("Unexpected size");
        unsafe { std::mem::transmute::<[u8; 8], u64>(arr) }
    }

    pub fn as_unibble_bit_vector(&self) -> BitVector {
        BitVector {
            bits: Self::fill_until_size(self.bits.clone(), 4, false)
                .into_iter()
                .take(4)
                .collect(),
        }
    }

    fn to_bit_string(bits: [bool; 8]) -> String {
        let strs: Vec<&str> = bits
            .clone()
            .iter()
            .map(|bit: &bool| if *bit { "1" } else { "0" })
            .collect();
        strs.join("")
    }

    fn as_byte_chunks(bits: Vec<bool>) -> Vec<[bool; 8]> {
        Self::fill_until_size(
            bits.clone(),
            ((bits.len() as f32) / 8.0).ceil() as usize * 8,
            false,
        )
        .chunks(8)
        .map(|x| x.try_into().expect("Chunk doesn't have 8 bits"))
        .collect()
    }

    fn fill_until_size<T: Copy>(vec: Vec<T>, size: usize, with: T) -> Vec<T> {
        let mut mut_vec = vec;
        while mut_vec.len() < size {
            mut_vec.push(with);
        }
        mut_vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bit_vector_generates_empty_byte_array_if_empty() {
        let bit_vector = BitVector { bits: vec![] };
        let u8_array = bit_vector.as_u8_array();

        assert!(u8_array.is_empty());
    }
}
