# Digital Fax

This is a submission for the Internet Communication software project summer term 2022 at FU.

## Setup

### Cross Compilation

In order to cross-compile for ARM, you need to install a GCC compiler for ARM. On macOS this can be done using this GitHub repo: [macos-cross-toolchains](https://github.com/messense/homebrew-macos-cross-toolchains).
The compiler will probably have a name similar to `arm-unknown-linux-gnueabihf-gcc` and the archiver `arm-unknown-linux-gnueabihf-ar` but check this for your specific installation.

Compile display component static library for ARM:

```sh
cd display
make lib CC=<COMPILER_NAME> AR=<ARCHIVER_NAME>
```

Compile management component for ARM:

```sh
cargo build --release --target arm-unknown-linux-gnueabihf
cargo build --release --target arm-unknown-linux-gnueabihf --features display
```
