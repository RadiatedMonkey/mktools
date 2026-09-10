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

    fn decode(reader: &mut Cursor<&[u8]>) -> Self::Output;
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

    fn decode(reader: &mut Cursor<&[u8]>) -> Self::Output {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yaz0File {
    header: Yaz0Header,
    uncompressed: Vec<u8>,
}

impl Decode for Yaz0File {
    type Output = Yaz0Result<Self>;

    fn decode(reader: &mut Cursor<&[u8]>) -> Self::Output {
        let header = Yaz0Header::decode(reader)?;
        let mut uncompressed = Vec::with_capacity(header.uncompressed_size as usize);
        let compressed = &reader.get_ref()[reader.position() as usize..];

        // Amount of chunks in a data group
        const CHUNK_COUNT: usize = 8;

        let mut block_count = 0;
        while uncompressed.len() < uncompressed.capacity() {
            let mut group_header = reader.read_u8()?;
            for _ in 0..CHUNK_COUNT {
                if uncompressed.len() >= uncompressed.capacity() {
                    break;
                }

                // If the bit is set, the chunk is 1 byte.
                // Otherwise it is 2 or 3 bytes
                let bit = (group_header & 0x80) != 0;
                group_header <<= 1;

                if bit {
                    // Copy over 1 byte immediately
                    uncompressed.push(reader.read_u8()?);
                } else {
                    // Perform run-length decoding.
                    // Bytes either look like
                    // NR RR or 0R RR NN
                    //
                    // RRR is a value between 0x000 and 0xfff that specifies the source location of the stream.
                    // For the first case, SIZE = N + 2 while for the second case SIZE = N + 0x12.
                    //
                    // A chunk may also reference itself.

                    // Read first two bytes of chunk
                    let b1 = reader.read_u8()? as usize;
                    let b2 = reader.read_u8()? as usize;

                    let rrr = (b1 & 0x0f) << 8 | b2;

                    let mut n = b1 >> 4;
                    let copy_size;

                    if n == 0 {
                        // 3 byte data, NN is at the end
                        n = reader.read_u8()? as usize;
                        copy_size = n + 0x12;
                    } else {
                        copy_size = n + 2;
                    }

                    let copy_start = uncompressed.len().checked_sub(rrr + 1).ok_or_else(|| {
                        Yaz0Error::InvalidFile(
                            "data group references byte before start of file".to_owned(),
                        )
                    })?;
                    let copy_end = copy_start + copy_size;

                    for _ in 0..copy_size {
                        let b = uncompressed[copy_start];
                        uncompressed.push(b);
                    }

                    block_count += 1;
                }
            }
        }

        dbg!(block_count);

        Ok(Self {
            header,
            uncompressed,
        })
    }
}

impl Yaz0File {
    pub fn read<P: AsRef<Path>>(path: P) -> Yaz0Result<Self> {
        let raw_file = std::fs::read(path.as_ref())?;
        let mut cursor = Cursor::new(raw_file.as_slice());

        let yaz0 = Yaz0File::decode(&mut cursor)?;

        // println!("{yaz0:?}");

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
