use std::io::Cursor;

use crate::{brres::Subfile, error::EncodingResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pat0Subfile {}

impl Pat0Subfile {
    pub fn decode(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        todo!()
    }
}

impl Subfile for Pat0Subfile {
    const MAGIC: [u8; 4] = [0x50, 0x41, 0x54, 0x30]; // "PAT0"
}
