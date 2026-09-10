use std::{
    io::{BufReader, Cursor},
    os::windows::raw,
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error;

macro_rules! impl_read_array {
    ($($ty: ty),*) => {
        paste::paste! {
            pub trait ReadArrayExt {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]>;
                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]>;

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>;
                )*
            }

            impl<R: ReadBytesExt> ReadArrayExt for R {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_u8()?;
                    }

                    Ok(array)
                }

                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_i8()?;
                    }

                    Ok(array)
                }

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>
                    {
                        let mut array = [0; N];
                        for i in 0..N {
                            array[i] = self.[< read_ $ty >]::<B>()?;
                        }

                        Ok(array)
                    }
                )*
            }
        }
    }
}

pub const YAZ0_MAGIC: &[u8] = "Yaz0".as_bytes();

impl_read_array!(u16, i16, u32, i32, u64, i64, u128, i128);

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Yaz0Error {
    #[error("invalid file: {0}")]
    InvalidFile(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unknown: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for Yaz0Error {
    fn from(value: std::io::Error) -> Self {
        use std::io::ErrorKind;

        match value.kind() {
            ErrorKind::InvalidData => Self::InvalidFile(value.to_string()),
            ErrorKind::NotFound => Self::NotFound(value.to_string()),
            _ => Self::Unknown(value.to_string()),
        }
    }
}

pub type Yaz0Result<T> = Result<T, Yaz0Error>;

pub trait Decode {
    type Output;

    fn decode<R: ReadBytesExt>(reader: &mut R) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yaz0Header {
    /// Magic of the file, must always equal "Yaz0" in ASCII.
    pub magic: [u8; 4],
    /// Size in bytes of the uncompressed file.
    pub uncompressed_size: u32,
    /// Reserved for special use. Always 0 in Mario Kart Wii.
    pub reserved: [u32; 2],
}

impl Decode for Yaz0Header {
    type Output = Yaz0Result<Self>;

    fn decode<R: ReadBytesExt>(reader: &mut R) -> Self::Output {
        let magic = reader.read_u8_array::<4>()?;
        if magic != YAZ0_MAGIC {
            return Err(Yaz0Error::InvalidFile(
                "Yaz0 magic does not equal `Yaz0`".to_owned(),
            ));
        }

        let uncompressed_size = reader.read_u32::<BigEndian>()?;
        let reserved = reader.read_u32_array::<2, BigEndian>()?;

        if reserved != [0, 0] {
            tracing::warn!("`reserved` field in Yaz0 header is not all zeros");
        }

        Ok(Self {
            magic,
            uncompressed_size,
            reserved,
        })
    }
}

pub struct DataGroup {
    pub header: u8,
}

impl Decode for DataGroup {
    type Output = Yaz0Result<Self>;

    fn decode<R: ReadBytesExt>(reader: &mut R) -> Self::Output {
        let mut header = reader.read_u8()?;
        for _ in 0..8 {
            // If the bit is set, the chunk is 1 byte.
            // Otherwise it is 2 or 3 bytes
            let bit = (header & 0x80) != 0;
            header <<= 1;

            if bit {
            } else {
            }
        }

        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yaz0File {
    header: Yaz0Header,
}

impl Decode for Yaz0File {
    type Output = Yaz0Result<Self>;

    fn decode<R: ReadBytesExt>(reader: &mut R) -> Self::Output {
        let header = Yaz0Header::decode(reader)?;

        Ok(Self { header })
    }
}

impl Yaz0File {
    pub fn read<P: AsRef<Path>>(path: P) -> Yaz0Result<Self> {
        let mut raw_file = Cursor::new(std::fs::read(path.as_ref())?);
        let yaz0 = Yaz0File::decode(&mut raw_file)?;

        dbg!(yaz0);

        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::yaz0::Yaz0File;

    #[test]
    fn read_yaz0() {
        let yaz0 = Yaz0File::read("test/fk-7-allkart.szs").unwrap();
    }
}
