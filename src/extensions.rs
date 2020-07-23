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

// To insert easily side effects in flows anywhere (from Kotlin)
pub trait Also: Sized {
    fn also<T>(self, f: T) -> Self
    where
        T: FnOnce(&Self) -> (),
    {
        f(&self);
        self
    }
}

impl<T> Also for T {}
