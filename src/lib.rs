pub mod arc;
pub mod brres;
pub mod chr0;
pub mod encoding;
pub mod error;
pub mod mdl0;
pub mod pat0;
pub mod yaz0;

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[cfg(test)]
mod test {
    use crate::{arc, brres, encoding::Decode, yaz0::decompress_yaz0};
    use std::io::Cursor;

    #[test]
    fn read_yaz0_arc() {
        crate::setup_tracing();

        let raw_yaz0 = std::fs::read("test/fk-7-allkart.szs").unwrap();
        let decompressed = decompress_yaz0(&raw_yaz0).unwrap();

        let mut cursor = Cursor::new(decompressed.as_slice());
        let mut arc = arc::Archive::decode(&mut cursor).unwrap();

        let node = arc.nodes.remove(2);
        tracing::debug!("Opening node {}", node.name);

        let arc::NodeType::File { content, .. } = node.data else {
            panic!("second node is not a file")
        };

        std::fs::write("./dump.bin", &content).unwrap();

        let mut cursor = Cursor::new(content.as_slice());
        let mut brres = brres::Archive::decode(&mut cursor).unwrap();
    }
}
