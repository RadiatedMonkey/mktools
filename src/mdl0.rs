use std::io::Cursor;

use crate::{
    brres::{Subfile, SubfileHeader, SubfileType},
    encoding::Decode,
    error::EncodingResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mdl0Subfile {
    pub header: SubfileHeader,
}

impl Decode for Mdl0Subfile {
    fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        tracing::trace!("Reading MDL0 file");

        let header = SubfileHeader::decode(reader, SubfileType::Mdl0)?;

        todo!()
    }
}

impl Subfile for Mdl0Subfile {
    const MAGIC: [u8; 4] = [0x4d, 0x44, 0x4c, 0x30]; // "MDL0"
}
