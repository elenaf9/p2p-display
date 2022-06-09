use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/control_message.proto"], &["src/"])?;

    if cfg!(feature = "display") {
        println!("cargo:rerun-if-changed=../display/libdisplay.a");
        println!("cargo:rustc-link-search=../display");
        println!("cargo:rustc-link-lib=bcm2835");
    }
    Ok(())
}
