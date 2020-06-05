use crate::preferences::{Preferences, TckBytesWrapper, TCK_SIZE_IN_BYTES};
use std::{io::Cursor, sync::Arc};
use tcn::{
    Error, MemoType, ReportAuthorizationKey, SignedReport, TemporaryContactKey,
    TemporaryContactNumber,
};

pub trait TcnKeys {
    fn create_report(&self, report: Vec<u8>) -> Result<SignedReport, Error>;
    fn generate_tcn(&self) -> TemporaryContactNumber;
}

pub trait ReportAuthorizationKeyExt {
    fn with_bytes(bytes: [u8; 32]) -> ReportAuthorizationKey {
        ReportAuthorizationKey::read(Cursor::new(&bytes)).expect("Couldn't read RAK bytes")
    }
}

impl ReportAuthorizationKeyExt for ReportAuthorizationKey {}

pub trait TckBytesWrapperExt {
    fn with_bytes(bytes: Vec<u8>) -> TckBytesWrapper {
        let mut array = [0; TCK_SIZE_IN_BYTES];
        let bytes = &bytes[..array.len()]; // panics if not enough data
        array.copy_from_slice(bytes);
        TckBytesWrapper { tck_bytes: array }
    }
}

impl TckBytesWrapperExt for TckBytesWrapper {}

pub struct TcnKeysImpl<T>
where
    T: Preferences,
{
    pub preferences: Arc<T>,
}

impl<T> TcnKeys for TcnKeysImpl<T>
where
    T: Preferences,
{
    fn create_report(&self, report: Vec<u8>) -> Result<SignedReport, Error> {
        let end_index = self.tck().index();
        let periods = 14 * 24 * (60 / 15);
        let mut start_index = 1;
        if end_index > periods {
            start_index = (end_index - periods) as u16
        }
        println!("start_index={}, end_index={}", start_index, end_index);

        self.rak()
            .create_report(MemoType::CoEpiV1, report, start_index, end_index)
    }

    fn generate_tcn(&self) -> TemporaryContactNumber {
        let tck = self.tck();
        let tcn = tck.temporary_contact_number();
        let new_tck = tck.ratchet();

        if let Some(new_tck) = new_tck {
            self.set_tck(new_tck);
        }

        println!("RUST generated tcn: {:?}", tcn);
        // TODO: if None, rotate RAK
        tcn
    }
}

impl<T> TcnKeysImpl<T>
where
    T: Preferences,
{
    fn rak(&self) -> ReportAuthorizationKey {
        self.preferences
            .authorization_key()
            .map(|rak_bytes| ReportAuthorizationKey::with_bytes(rak_bytes)) //Self::bytes_to_rak(rak_bytes))
            .unwrap_or({
            .unwrap_or_else(|| {
                let new_key = ReportAuthorizationKey::new(rand::thread_rng());
                self.preferences
                    .set_autorization_key(Self::rak_to_bytes(new_key));
                new_key
            })
    }

    fn tck(&self) -> TemporaryContactKey {
        self.preferences
            .tck()
            .map(|tck_bytes| Self::bytes_to_tck(tck_bytes))
            .unwrap_or({ self.rak().initial_temporary_contact_key() })
    }

    fn set_tck(&self, tck: TemporaryContactKey) {
        self.preferences.set_tck(Self::tck_to_bytes(tck));
    }

    fn rak_to_bytes(rak: ReportAuthorizationKey) -> [u8; 32] {
        let mut buf = Vec::new();
        rak.write(Cursor::new(&mut buf))
            .expect("Couldn't write RAK bytes");
        Self::byte_vec_to_32_byte_array(buf)
    }

    fn byte_vec_to_32_byte_array(bytes: Vec<u8>) -> [u8; 32] {
        let mut array = [0; 32];
        let bytes = &bytes[..array.len()]; // panics if not enough data
        array.copy_from_slice(bytes);
        array
    }

    pub fn tck_to_bytes(tck: TemporaryContactKey) -> TckBytesWrapper {
        let mut buf = Vec::new();
        tck.write(Cursor::new(&mut buf))
            .expect("Couldn't write TCK bytes");
        // Self::byte_vec_to_tck_byte_wrapper(buf)
        TckBytesWrapper::with_bytes(buf)
    }

    fn bytes_to_tck(tck: TckBytesWrapper) -> TemporaryContactKey {
        TemporaryContactKey::read(Cursor::new(&tck)).expect("Couldn't read TCK bytes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preferences::PreferencesTckMock;

    #[test]
    fn test_rak() {
        let new_key = ReportAuthorizationKey::new(rand::thread_rng());
        let bytes = TcnKeysImpl::<PreferencesTckMock>::rak_to_bytes(new_key);
        println!("{:?}", bytes);
    }

    #[test]
    fn test_load_rak() {
        let bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];
        let key = ReportAuthorizationKey::with_bytes(bytes);
        let tck = key.initial_temporary_contact_key();
        TcnKeysImpl::<PreferencesTckMock>::tck_to_bytes(tck);
    }

    #[test]
    fn test_load_tck() {
        let rak_bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];
        let rak = ReportAuthorizationKey::with_bytes(rak_bytes);
        let _tck_1 = rak.initial_temporary_contact_key();

        let tck_inner_bytes = [
            34, 166, 47, 23, 224, 52, 240, 95, 140, 186, 95, 243, 26, 13, 174, 128, 224, 229, 158,
            248, 117, 7, 118, 110, 108, 57, 67, 206, 129, 22, 84, 13,
        ];
        println!("count = {}", tck_inner_bytes.len());

        let version_bytes: [u8; 2] = [1, 0];

        let version_vec = version_bytes.to_vec();
        let rak_vec = rak_bytes.to_vec();
        let tck_inner_vec = tck_inner_bytes.to_vec();

        let complete_tck_vec = [&version_vec[..], &rak_vec[..], &tck_inner_vec[..]].concat();

        let tck_bytes_wrapped = TckBytesWrapper::with_bytes(complete_tck_vec);
        let tck = TcnKeysImpl::<PreferencesTckMock>::bytes_to_tck(tck_bytes_wrapped);

        println!("{:#?}", tck);
    }

    #[test]
    fn test_generate_tcns() {
        let rak_bytes = [
            42, 118, 64, 131, 236, 36, 122, 23, 13, 108, 73, 171, 102, 145, 66, 91, 157, 105, 195,
            126, 139, 162, 15, 31, 0, 22, 31, 230, 242, 241, 225, 85,
        ];

        let rak = ReportAuthorizationKey::with_bytes(rak_bytes);
        let mut tck = rak.initial_temporary_contact_key(); // tck <- tck_1
        let mut tcns = Vec::new();

        for _ in 0..100 {
            tcns.push(tck.temporary_contact_number());
            tck = tck.ratchet().unwrap();
        }
    }
}
