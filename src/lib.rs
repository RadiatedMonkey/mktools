pub mod arc;
pub mod encoding;
pub mod error;
pub mod yaz0;

#[cfg(test)]
mod test {
    use crate::{arc::ArcFile, encoding::Decode, yaz0::decompress_yaz0};
    use std::io::Cursor;

    #[test]
    fn read_yaz0_arc() {
        let raw_yaz0 = std::fs::read("test/fk-7-allkart.szs").unwrap();
        let decompressed = decompress_yaz0(&raw_yaz0).unwrap();

        let mut cursor = Cursor::new(decompressed.as_slice());
        let arc = ArcFile::decode(&mut cursor).unwrap();
    }
}
