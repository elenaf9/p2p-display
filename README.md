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

### Raspberry Pi

To set up a new Raspberry Pi, install your SSH key on it and then:

```sh
export CC=<COMPILER_NAME> && export AR=<ARCHIVER_NAME>
./scripts/build.sh # if you want to compile with display support, add --features display
./scripts/setup.sh <HOSTNAME_PI>
```

## Scripts

There are some helper scripts to make stuff easier:

**Build for ARM with display support**: `./scripts/build.sh --features display`. (If you want to use a custom compiler and archiver use `export CC=<COMPILER_NAME> && export AR=<ARCHIVER_NAME>` before running the build script.)

**Setup a new Raspberry Pi**: `./scripts/setup.sh <HOSTNAME_PI>`
