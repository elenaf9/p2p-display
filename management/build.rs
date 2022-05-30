use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/control_message.proto"], &["src/"])?;

    if cfg!(feature = "display") {
        cc::Build::new()
            .file("../display/toDisplay.c")
            .compile("toDisplay");
    }
    Ok(())
}
