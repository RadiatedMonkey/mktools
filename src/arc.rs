use std::{ffi::CStr, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    encoding::{Decode, Encode, ReadArrayExt, WriteArrayExt},
    error::{EncodingError, EncodingResult},
};

/// Magic of an ARC file.
const ARC_MAGIC: u32 = 0x55AA382D;

/// See [`Custom Mario Kart Wiiki`](https://mkwiiki.org/wiki/ARC_(File_Format)) for more info.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    /// Offset to the first node in the archive.
    pub node_offset: i32,
    /// Size of all nodes including the string table.
    pub size: i32,
    /// File offset of data.
    pub file_offset: i32,
    /// Reserved.
    pub reserved: [i32; 4],
}

impl Decode for Header {
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

impl Encode for Header {
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
enum RawNodeType {
    File,
    Directory,
}

impl Decode for RawNodeType {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }
}

impl Encode for RawNodeType {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl TryFrom<u8> for RawNodeType {
    type Error = EncodingError;

    fn try_from(value: u8) -> EncodingResult<Self> {
        Ok(match value {
            0 => RawNodeType::File,
            1 => RawNodeType::Directory,
            v => {
                return Err(EncodingError::InvalidFile(format!(
                    "arc node type is expected to be either 0 (file) or 1 (directory), got {v}"
                )));
            }
        })
    }
}

impl From<&NodeType> for RawNodeType {
    fn from(value: &NodeType) -> Self {
        match value {
            NodeType::File { .. } => RawNodeType::File,
            NodeType::Directory { .. } => RawNodeType::Directory,
        }
    }
}

/// Exact size of a single ARC node.
const ARC_NODE_SIZE: usize = 0x0c;

/// Raw ARC node directly from the ARC file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RawNode {
    /// The type this node is.
    pub ty: RawNodeType,
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

impl RawNode {
    pub fn get_name_from_pool<'pool>(
        &self,
        string_pool: &'pool [u8],
    ) -> EncodingResult<&'pool str> {
        let null_pos = string_pool[self.pool_offset as usize..]
            .iter()
            .position(|&b| b == 0x00)
            .unwrap_or(string_pool.len() - self.pool_offset as usize);

        Ok(str::from_utf8(
            &string_pool[self.pool_offset as usize..self.pool_offset as usize + null_pos],
        )?)
    }
}

impl Encode for RawNode {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        self.ty.encode_into(writer)?;
        writer.write_u24::<BigEndian>(self.pool_offset)?;
        writer.write_u32::<BigEndian>(self.data1)?;
        writer.write_u32::<BigEndian>(self.data2)?;

        Ok(())
    }
}

impl Decode for RawNode {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let ty = RawNodeType::decode(reader)?;
        let pool_offset = reader.read_u24::<BigEndian>()?;
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
pub enum NodeType {
    File {
        /// Size of the file in bytes.
        size: u32,
        content: Vec<u8>,
    },
    Directory {
        /// Index of the parent directory.
        parent: u32,
        /// Index of the first node that is not part of this directory.
        skip_node: u32,
    },
}

/// Processed ARC node that now includes its name and other properties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub data: NodeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub nodes: Vec<Node>,
}

impl Archive {
    /// Sorts all files in alphabetical order within their directory.
    pub fn sort(&mut self) {
        tracing::trace!("Sorting {} ARC nodes", self.nodes.len());

        todo!()
    }
}

impl Encode for Archive {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        tracing::trace!("Encoding {} ARC nodes", self.nodes.len());

        let spool_size = 0;
        let fpool_offset = 0;

        let size = ARC_NODE_SIZE * self.nodes.len() + spool_size;
        let header = Header {
            node_offset: ARC_NODE_SIZE as i32,
            file_offset: fpool_offset,
            size: size as i32,
            reserved: [0; 4],
        };

        header.encode_into(writer)?;

        for node in &self.nodes {
            // let raw_node = RawNode {
            //     ty: RawNodeType::from(&node.data),
            // };
            let raw_node: RawNode = todo!();

            raw_node.encode_into(writer)?;
        }

        todo!();

        Ok(())
    }
}

impl Decode for Archive {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let header = Header::decode(reader)?;
        let root_node = RawNode::decode(reader)?;
        let node_count = root_node.data2;
        tracing::trace!("Reading {node_count} ARC nodes");

        // The string pool starts right after the last node.
        let spool_start = header.node_offset as usize + ARC_NODE_SIZE * node_count as usize;
        let spool_end = header.node_offset as usize + header.size as usize;
        tracing::trace!("ARC string pool range is {spool_start}...{spool_end}");

        let spool = &reader.get_ref()[spool_start..spool_end];

        let mut raw_nodes = Vec::with_capacity(node_count as usize);
        raw_nodes.push(root_node);

        for _ in 1..node_count {
            let node = RawNode::decode(reader)?;
            raw_nodes.push(node);
        }

        tracing::trace!("Loading ARC node names and content...");

        // Process all nodes to find their data and names.
        let mut nodes = Vec::with_capacity(raw_nodes.len());
        for raw_node in raw_nodes {
            let name = raw_node.get_name_from_pool(spool)?.to_owned();

            let reader_buf = reader.get_ref();
            let data = match raw_node.ty {
                RawNodeType::File => {
                    let data_start = raw_node.data1 as usize;
                    let data_end = data_start + raw_node.data2 as usize;

                    tracing::trace!(
                        "Discovered file node `{name}` with data section {data_start}...{data_end}"
                    );

                    if data_end > reader_buf.len() {
                        return Err(EncodingError::InvalidFile(format!(
                            "file node data range {data_start}..{data_end} exceeds file length {}",
                            reader_buf.len()
                        )));
                    }

                    NodeType::File {
                        content: reader_buf[data_start..data_end].to_owned(),
                        size: raw_node.data2,
                    }
                }
                RawNodeType::Directory => {
                    tracing::trace!(
                        "Discovered directory node `{name}` (parent node: {}, end node: {})",
                        raw_node.data1,
                        raw_node.data2
                    );

                    NodeType::Directory {
                        parent: raw_node.data1,
                        skip_node: raw_node.data2,
                    }
                }
            };

            nodes.push(Node { name, data })
        }

        debug_assert_eq!(
            reader.position(),
            spool_start as u64,
            "not all bytes of the arc file were read"
        );

        tracing::trace!("Successfully loaded node names and data pointers");

        Ok(Self { nodes })
    }
}
