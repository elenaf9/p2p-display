# P2P File sharing application using IPFS

**Idea:** Build application for local, on-demand file sharing between a group of peers

- (Not directly an IoT application, but still belongs of the large field of internet-communication)
- Enable  peer-to-peer file sharing between a group of people using IPFS: <https://docs.ipfs.io/concepts/what-is-ipfs/>
- Content-bases addressing (see below) allows other peers to also provide the same file even if the original provider already left the group
- No central server needed, p2p-network can be created on demand in the same local network

## Technical Details

- Use go-IPFS: <https://github.com/ipfs/go-ipfs> (most mature IPFS implementation)
  - Content-based addressing: <https://docs.ipfs.io/concepts/what-is-ipfs/#content-addressing>
  - Build with libp2p
  - -> In itself this application is rather simple to implement, but can be extended (see below)

## Use cases

- Sharing files between a group of people in the same network; anybody can provide all the files at any time

## Possible Extensions

- Instead of using the IPFS implementation in `Go` directly, use rust-ipfs/ js-ipfs (both not very mature) or build on an libp2p implementation directly
  - libp2p is implement in Go, Rust, JS, JVM, Python and Erlang (and some more I think)
  - libp2p implements the kademlia DHT protocol that IPFS uses: <https://www.scs.stanford.edu/~dm/home/papers/kpos.pdf>
  - -> would be more focuses on IPFS itself rather than a specific use-case
- Enable connecting to peers outside of the local network using hole-punching <https://blog.ipfs.io/2022-01-20-libp2p-hole-punching/>
- Enable integrating a printer to the network to print a file
- .. other use-cases for connecting IoT devices to the network?

## Ressources
- IPFS: <https://github.com/ipfs/ipfs>
- Libp2p (general docs): <https://libp2p.io/>
  - rust-libp2p: <https://github.com/libp2p/rust-libp2p>
  - js-libp2p: <https://github.com/libp2p/js-libp2p>,
  - python-libp2p: <https://github.com/libp2p/py-libp2p>
  - jvm-libp2p: <https://github.com/libp2p/jvm-libp2p>
  - erlang-libp2p: <https://github.com/helium/erlang-libp2p>
- Kademlia DHT: <https://www.scs.stanford.edu/~dm/home/papers/kpos.pdf>
