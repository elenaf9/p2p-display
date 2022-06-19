# Digital Fax

This is a submission for the Internet Communication software project summer term 2022 at FU.

## Setup

### Cross Compilation

In order to cross-compile for arm, you need to install a GCC compiler for arm. On macOS this can be done using this GitHub repo: [macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains).

To compile with display support, you need to build the display library on a Raspberry Pi using the instructions from the display directory. The pre-built `libdisplay.a` must be copied to the display directory.

Compile for ARM:

```sh
cargo build --release --target arm-unknown-linux-gnueabihf
cargo build --release --target arm-unknown-linux-gnueabihf --features display
```
