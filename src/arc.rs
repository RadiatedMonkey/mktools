use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    encoding::{Decode, Encode, ReadArrayExt, WriteArrayExt},
    error::{EncodingError, EncodingResult},
};

pub const ARC_MAGIC: u32 = 0x55AA382D;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcHeader {
    pub node_offset: i32,
    pub size: i32,
    pub file_offset: i32,
    pub reserved: [i32; 4],
}

impl Decode for ArcHeader {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u32::<BigEndian>()?;
        if magic != ARC_MAGIC {
            return Err(EncodingError::InvalidFile(
                "ARC file magic is incorrect".to_owned(),
            ));
        }

        let node_offset = reader.read_i32::<BigEndian>()?;
        let size = reader.read_i32::<BigEndian>()?;
        let file_offset = reader.read_i32::<BigEndian>()?;
        let reserved = reader.read_i32_array::<4, BigEndian>()?;

        Ok(Self {
            node_offset,
            size,
            file_offset,
            reserved,
        })
    }
}

impl Encode for ArcHeader {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u32::<BigEndian>(ARC_MAGIC)?;
        writer.write_i32::<BigEndian>(self.node_offset)?;
        writer.write_i32::<BigEndian>(self.size)?;
        writer.write_i32::<BigEndian>(self.file_offset)?;
        writer.write_i32_array::<4, BigEndian>(self.reserved)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcFile {
    pub header: ArcHeader,
}

impl Decode for ArcFile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        todo!()
    }
}
