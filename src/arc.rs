use std::{ffi::CStr, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    encoding::{Decode, Encode, ReadArrayExt, WriteArrayExt},
    error::{EncodingError, EncodingResult},
};

/// Magic of an ARC file.
pub const ARC_MAGIC: u32 = 0x55AA382D;

/// See [`Custom Mario Kart Wiiki`](https://mkwiiki.org/wiki/ARC_(File_Format)) for more info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcHeader {
    /// Offset to the first node in the archive.
    pub node_offset: i32,
    /// Size of all nodes including the string table.
    pub size: i32,
    /// File offset of data.
    pub file_offset: i32,
    /// Reserved.
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ArcNodeType {
    File,
    Directory,
}

impl Decode for ArcNodeType {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }
}

impl Encode for ArcNodeType {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl TryFrom<u8> for ArcNodeType {
    type Error = EncodingError;

    fn try_from(value: u8) -> EncodingResult<Self> {
        Ok(match value {
            0 => ArcNodeType::File,
            1 => ArcNodeType::Directory,
            v => {
                return Err(EncodingError::InvalidFile(format!(
                    "arc node type is expected to be either 0 (file) or 1 (directory), got {v}"
                )));
            }
        })
    }
}

/// Exact size of a single ARC node.
pub const ARC_NODE_SIZE: usize = 0x0c;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcNode {
    /// The type this node is.
    pub ty: ArcNodeType,
    /// Offset into the string pool for the file name.
    pub pool_offset: u32,
    /// Data content depends on the node type:
    /// - File: Offset of begin of data.
    /// - Directory: Index of the parent directory.
    pub data1: u32,
    /// Data content depends on the node type:
    /// - File: Size of data
    /// - Directory: Index of the first node that is not part of this directory.
    pub data2: u32,
}

impl ArcNode {
    pub fn get_node_name<'pool>(&self, string_pool: &'pool [u8]) -> EncodingResult<&'pool str> {
        let null_pos = string_pool[self.pool_offset as usize..]
            .iter()
            .position(|&b| b == 0x00)
            .unwrap_or(string_pool.len() - self.pool_offset as usize);

        dbg!(self.pool_offset);
        dbg!(null_pos);

        dbg!(String::from_utf8_lossy(
            &string_pool[self.pool_offset as usize..self.pool_offset as usize + null_pos]
        ));

        Ok(str::from_utf8(
            &string_pool[self.pool_offset as usize..self.pool_offset as usize + null_pos],
        )?)
    }
}

impl Encode for ArcNode {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        self.ty.encode_into(writer)?;
        writer.write_u24::<BigEndian>(self.pool_offset)?;
        writer.write_u32::<BigEndian>(self.data1)?;
        writer.write_u32::<BigEndian>(self.data2)?;

        Ok(())
    }
}

impl Decode for ArcNode {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        println!(
            "{:?}",
            &reader.get_ref()
                [reader.position() as usize..reader.position() as usize + ARC_NODE_SIZE]
        );

        let ty = ArcNodeType::decode(reader)?;

        // dbg!(&reader.get_ref()[reader.position() as usize..reader.position() as usize + 3]);
        let pool_offset = reader.read_u24::<BigEndian>()?;
        dbg!(pool_offset);

        let data1 = reader.read_u32::<BigEndian>()?;
        let data2 = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            ty,
            pool_offset,
            data1,
            data2,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArcFile {
    pub header: ArcHeader,
    pub nodes: Vec<ArcNode>,
}

impl ArcFile {}

impl Decode for ArcFile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let header = ArcHeader::decode(reader)?;
        let root_node = ArcNode::decode(reader)?;
        let node_count = root_node.data2;
        dbg!(node_count);

        let spool_start = header.node_offset as usize + ARC_NODE_SIZE * node_count as usize;
        let spool_end = header.node_offset as usize + header.size as usize;
        let spool = &reader.get_ref()[spool_start..spool_end];

        dbg!(spool_start, spool_end);

        dbg!(root_node.get_node_name(spool)?);

        let mut nodes = Vec::with_capacity(node_count as usize);
        nodes.push(root_node);

        for _ in 1..node_count {
            let start_pos = reader.position();

            let node = ArcNode::decode(reader)?;
            // dbg!(node.get_node_name(spool)?);

            nodes.push(node);

            let end_pos = reader.position();
            println!("read {}", end_pos - start_pos);
        }

        Ok(Self { header, nodes })
    }
}
