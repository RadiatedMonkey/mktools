use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    encoding::{Decode, ReadArrayExt},
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

pub struct RootSubfile {
    pub size: u32,
}

impl Decode for RootSubfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
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

pub struct ModelSubfile {
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

impl Decode for ModelSubfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != Self::MAGIC {
            return Err(EncodingError::InvalidFile(
                "incorrect MDL0 file magic".to_owned(),
            ));
        }

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

impl Subfile for ModelSubfile {
    const MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"
}

macro_rules! impl_subfile_enum {
    ($($ty: ident),*) => {
        paste::paste! {
            pub enum SubfileType {
                $($ty),*
            }

            pub enum SubfileData {
                $($ty([< $ty Subfile >])),*
            }
        }
    }
}

impl_subfile_enum!(Root, Model);

pub struct Archive {
    pub sections: Vec<SubfileData>,
}

impl Decode for Archive {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let root_subfile = RootSubfile::decode(reader)?;

        todo!()
    }
}
