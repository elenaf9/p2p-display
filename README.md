# Digital Fax

_WIP project for the internet-communication course summer term 2022 at FU Berlin._

**Idea**: Control the content displayed on a E-Ink Display from remote via P2P Communication
- E-ink display connected to an IoT device that can write to the display
- Setup P2P node on the IoT device, nodes are connected in a mesh-network
- Displayed content can be changed by other (authorized) peers by sending messages to the node 
- E-ink display has very low energy consumption - can be unplugged any time and will continue displaying content

## Components

### Display

Displays a message on an E-Ink display:
- Interface with the display via SPI
- Uses bcm2835 driver files to access GPIO 

### Network

Responsible for sending and receiving data within the network:
- P2P network written in Rust using libp2p
- Peer discovery via MDNS, Pub/Sub communication via GossipSub
- Nodes connected in a mesh-network using B.A.T.M.A.N

### Management

Interface between network, display and user:
- Embeds network component as library and display component via FFI.
- Implements protocol for messages, encoded with protobuf
- Authenticates messages

## Hardware Setup

- Raspberry Pi 3/4
- Waveshare e-Paper Module

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
