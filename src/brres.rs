use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    encoding::{Decode, Encode, ReadArrayExt},
    error::{EncodingError, EncodingResult},
};

const BRRES_MAGIC: [u8; 4] = [0x62, 0x72, 0x65, 0x73];
const LE_BOM: [u8; 2] = [0xFF, 0xFE];
const BE_BOM: [u8; 2] = [0xFE, 0xFF];

/// Returns the amount of sections a subfile has, which depends on the subfile type and version.
fn get_section_count(ty: SubfileType, version: u32) -> EncodingResult<usize> {
    Ok(match ty {
        SubfileType::Root => 0,
        SubfileType::Model => match version {
            8 => 11,
            11 => 14,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid MDL0 version: {version} (must be 8, 11)"
                )));
            }
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    pub size: u32,
    pub root_offset: u16,
    pub section_count: u16,
}

impl Decode for Header {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != BRRES_MAGIC {
            return Err(EncodingError::InvalidFile(
                "brres file does not have the correct magic".to_owned(),
            ));
        }

        let is_be = match reader.read_u8_array::<2>()? {
            BE_BOM => true,
            LE_BOM => false,
            bom => {
                return Err(EncodingError::InvalidFile(format!(
                    "byte order mark is incorrect, expected FEFF or FFFE, found {bom:x?}"
                )));
            }
        };

        if !is_be {
            return Err(EncodingError::Unsupported(
                "little endian brres files are not supported".to_owned(),
            ));
        }

        let _padding = reader.read_u16::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;
        let root_offset = reader.read_u16::<BigEndian>()?;
        let section_count = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            root_offset,
            size,
            section_count,
        })
    }
}

pub trait Subfile {
    const MAGIC: [u8; 4];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSubfile {
    pub size: u32,
}

impl Decode for RootSubfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        println!("{magic:?}");

        if magic != Self::MAGIC {
            return Err(EncodingError::InvalidFile(
                "root subfile has incorrect magic".to_owned(),
            ));
        }

        Ok(RootSubfile {
            size: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl Subfile for RootSubfile {
    const MAGIC: [u8; 4] = [0x72, 0x6f, 0x6f, 0x74]; // "root"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubfileHeader {
    /// Length of this subfile.
    pub subfile_length: u32,
    /// Version of ths subfile. For MDL0 this is either 8 or 11.
    pub subfile_version: u32,
    /// The offset to the outer BRRES file.
    pub brres_offset: i32,
    /// Offsets within this BRRES file. The number of offsets is implied by the version.
    /// The number can be obtained using the [`get_section_count`] function.
    pub offsets: Vec<i32>,
    /// String offset to the name of this subfile.
    pub name_offset: i32,
}

impl Decode for SubfileHeader {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let subfile_length = reader.read_u32::<BigEndian>()?;
        let subfile_version = reader.read_u32::<BigEndian>()?;
        let brres_offset = reader.read_i32::<BigEndian>()?;

        let section_count = get_section_count(SubfileType::Model, subfile_version)?;

        let mut offsets = Vec::with_capacity(section_count);
        for _ in 0..section_count {
            offsets.push(reader.read_i32::<BigEndian>()?);
        }

        let name_offset = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            subfile_length,
            subfile_version,
            brres_offset,
            offsets,
            name_offset,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSubfile {
    pub header: SubfileHeader,
}

impl Decode for ModelSubfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != Self::MAGIC {
            return Err(EncodingError::InvalidFile(
                "incorrect MDL0 file magic".to_owned(),
            ));
        }

        let header = SubfileHeader::decode(reader)?;

        todo!()
    }
}

impl Subfile for ModelSubfile {
    const MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationPolicy {
    OneTime,
    Loop,
}

impl TryFrom<u32> for AnimationPolicy {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::OneTime,
            0x01 => Self::Loop,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid animation policy: {value} (expected 0 or 1)"
                )));
            }
        })
    }
}

impl Decode for AnimationPolicy {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let policy = reader.read_u32::<BigEndian>()?;
        Self::try_from(policy)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ScalingRule {
    Standard,
    Softimage,
    Maya,
}

impl TryFrom<u32> for ScalingRule {
    type Error = EncodingError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Standard,
            0x01 => Self::Softimage,
            0x02 => Self::Maya,
            _ => {
                return Err(EncodingError::InvalidFile(format!(
                    "invalid scaling rule: {} (expected 0, 1 or 2)",
                    value
                )));
            }
        })
    }
}

impl Decode for ScalingRule {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let rule = reader.read_u32::<BigEndian>()?;
        Self::try_from(rule)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chr0Header {
    pub frame_count: u16,
    pub anim_data_count: u16,
    pub anim_policy: AnimationPolicy,
    pub scaling_rule: ScalingRule,
}

impl Decode for Chr0Header {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        let anim_data_count = reader.read_u16::<BigEndian>()?;
        let anim_policy = AnimationPolicy::decode(reader)?;
        let scaling_rule = ScalingRule::decode(reader)?;

        Ok(Self {
            frame_count,
            anim_data_count,
            anim_policy,
            scaling_rule,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chr0Subfile {
    pub subfile_header: SubfileHeader,
    pub chr0_header: Chr0Header,
}

impl Decode for Chr0Subfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != Self::MAGIC {
            return Err(EncodingError::InvalidFile(
                "incorrect CHR0 file magic".to_owned(),
            ));
        }

        let subfile_header = SubfileHeader::decode(reader)?;
        let chr0_header = Chr0Header::decode(reader)?;

        dbg!(subfile_header, chr0_header);

        todo!()
    }
}

impl Subfile for Chr0Subfile {
    const MAGIC: [u8; 4] = [0x43, 0x48, 0x52, 0x30]; // "CHR0"
}

macro_rules! impl_subfile_enum {
    ($($ty: ident),*) => {
        paste::paste! {
            #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
            pub enum SubfileType {
                $($ty),*
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum SubfileData {
                $($ty([< $ty Subfile >])),*
            }
        }
    }
}

impl_subfile_enum!(Root, Model);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupHeader {
    pub length: u32,
    pub number: u32,
}

impl Decode for IndexGroupHeader {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        Ok(Self {
            length: reader.read_u32::<BigEndian>()?,
            number: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl Encode for IndexGroupHeader {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u32::<BigEndian>(self.length)?;
        writer.write_u32::<BigEndian>(self.number)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupEntry {
    pub entry_id: u16,
    pub flag: u16,
    pub left_index: u16,
    pub right_index: u16,
    pub name_pointer: i32,
    pub data_pointer: i32,
}

impl Decode for IndexGroupEntry {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let entry_id = reader.read_u16::<BigEndian>()?;
        let flag = reader.read_u16::<BigEndian>()?;
        let left_index = reader.read_u16::<BigEndian>()?;
        let right_index = reader.read_u16::<BigEndian>()?;
        let name_pointer = reader.read_i32::<BigEndian>()?;
        let data_pointer = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            entry_id,
            flag,
            left_index,
            right_index,
            name_pointer,
            data_pointer,
        })
    }
}

impl Encode for IndexGroupEntry {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u16::<BigEndian>(self.entry_id)?;
        writer.write_u16::<BigEndian>(self.flag)?;
        writer.write_u16::<BigEndian>(self.left_index)?;
        writer.write_u16::<BigEndian>(self.right_index)?;
        writer.write_i32::<BigEndian>(self.name_pointer)?;
        writer.write_i32::<BigEndian>(self.data_pointer)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroup {
    pub group_start: u32,
    pub header: IndexGroupHeader,
    pub entries: Vec<IndexGroupEntry>,
}

impl IndexGroup {
    pub fn get_entry_name<'pool>(
        &self,
        data: &'pool [u8],
        entry: &IndexGroupEntry,
    ) -> EncodingResult<&'pool str> {
        let name_buf = &data[self.group_start as usize + entry.name_pointer as usize..];

        dbg!(entry.name_pointer);

        let mut reader = Cursor::new(name_buf);
        let name_len = reader.read_u32::<BigEndian>()?;
        dbg!(name_len);

        let name = String::from_utf8_lossy(&name_buf[4..4 + name_len as usize]);
        dbg!(name);

        todo!()
    }
}

impl Decode for IndexGroup {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let group_start = reader.position() as u32;
        let header = IndexGroupHeader::decode(reader)?;

        let mut entries = Vec::with_capacity(header.number as usize);
        for _ in 0..header.number + 1 {
            let entry = IndexGroupEntry::decode(reader)?;
            println!("{entry:?}");
            entries.push(entry);
        }

        debug_assert_eq!(
            reader.position() as u32 - group_start,
            header.length,
            "not all index group entries were read"
        );

        Ok(Self {
            group_start,
            header,
            entries,
        })
    }
}

impl Encode for IndexGroup {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        self.header.encode_into(writer)?;
        for entry in &self.entries {
            entry.encode_into(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub sections: Vec<SubfileData>,
}

impl Decode for Archive {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let header = Header::decode(reader)?;

        println!("{header:?}");

        // Skip to root start
        reader.set_position(header.root_offset as u64);

        let root_subfile = RootSubfile::decode(reader)?;
        let index_group = IndexGroup::decode(reader)?;

        let root_index = &index_group.entries[1];
        let root_name = index_group
            .get_entry_name(&reader.get_ref()[header.root_offset as usize..], root_index)?;
        dbg!(root_name);

        todo!()
    }
}
