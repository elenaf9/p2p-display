use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/control_message.proto"], &["src/"])?;
    Ok(())
}
