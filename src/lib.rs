#![feature(error_generic_member_access)]

pub mod arc;
pub mod brres;
pub mod chr0;
pub mod encoding;
pub mod error;
pub mod mdl0;
pub mod pat0;
pub mod yaz0;

fn setup_tracing() {
    color_eyre::config::HookBuilder::new()
        .panic_section("boop")
        .add_frame_filter(Box::new(|frames| {
            let mut index = 0;
            frames.retain(|frame| {
                println!("{:?}", frame.name);
                true
            })
        }))
        .install()
        .unwrap();

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
        fn inner() -> color_eyre::Result<()> {
            crate::setup_tracing();

            let raw_yaz0 = std::fs::read("test/fk-7-allkart.szs")?;
            let decompressed = decompress_yaz0(&raw_yaz0)?;

            let mut cursor = Cursor::new(decompressed.as_slice());
            let mut arc = arc::Archive::decode(&mut cursor)?;

            let node = arc.nodes.remove(2);
            tracing::debug!("Opening node {}", node.name);

            let arc::NodeType::File { content, .. } = node.data else {
                panic!("second node is not a file")
            };

            std::fs::write("./dump.bin", &content)?;

            let mut cursor = Cursor::new(content.as_slice());
            let mut brres = brres::Archive::decode(&mut cursor)?;

            Ok(())
        }

        if let Err(err) = inner() {
            eprintln!("{err:#?}");
        }
    }
}
