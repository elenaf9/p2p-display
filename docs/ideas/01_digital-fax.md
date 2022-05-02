# "Digital Fax"

**Idea:** Control the content displayed on a E-Ink Display from remote via P2P Communication

- Have a E-Ink display connected to a smaller / larger IoT Device that controls the displayed content
- Setup a P2P node on the IoT Device that remote Peers can connect
- To change the displayed content, other (authorized) peers can send according messages to the node
- E-Ink display has very low energy consumption -  can be unplugged any time and will continue displaying content
- No central server needed, peers can easily join the network

## Technical Details

- P2P-Network written in Rust using [rust-libp2p](https://github.com/libp2p/rust-libp2p):
  - Supports TCP transport with multiplexing and encryption using the noise-protocol
  - *GossipSub* protocol for pub-sub communication in the P2P-Network
  - *Request-Response* protocol for 1:1 request-response messaging
  - Peer-discovery in a local network using multicast DNS
  - Rust can compile to a large number of different devices:
    - <https://doc.rust-lang.org/nightly/rustc/platform-support.html>
    - -> target needs to support the rust `std` library
- Simplest idea for the IoT device: Raspberry Pi -> Supported Target by Rust
- 

## Use Cases

- "Digital Fax" -  Display a message or pictures to a remote person; e.g. send pictures to the family from current vacation
- Shared document -  A shared document, e.g. Todo list, that hangs in the office (as an e-ink display) and anybody (authorized) can add points to the list
- Scheduled Room occupation in T9 building:
  - Each room has a e-ink display next to the door displaying the scheduled courses in it
  - Last minute changes can easily send by any prof to the display (as long as it is somehow connected to an online IoT device)
  - Alternatively: Have the e-ink display unplugged and only connect it to a IoT device on demand:
    - Small IoT device that can be plugged into a connector to the e-ink display
    - Connect with phone / laptop to the IoT device to change the content of the single display
- .. others
  
## Ressources

- Libp2p (general docs): <https://libp2p.io/>
- Connecting Rapsberry Pi to an E-Ink Display: <https://www.raspberrypi.com/news/using-e-ink-raspberry-pi/>

