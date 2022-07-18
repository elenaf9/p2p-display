use futures::{
    channel::{mpsc, oneshot},
    select, SinkExt, StreamExt,
};
use libp2p::{
    core,
    gossipsub::{
        error::PublishError, Gossipsub, GossipsubConfig, GossipsubEvent, GossipsubMessage,
        IdentTopic, MessageAuthenticity,
    },
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    mplex, noise,
    swarm::{dial_opts::DialOpts, SwarmEvent, DialError},
    tcp, yamux, Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport, request_response::{RequestResponse, RequestResponseConfig, RequestResponseEvent, ProtocolSupport, RequestResponseMessage},
};
use std::{collections::{HashMap, HashSet}, iter};

use crate::{NetworkEvent, protocol::{Codec, Protocol, Ack}};

// Fixed topic for the first PoC.
const TOPIC: &str = "topic";

pub enum Command {
    PublishMessage { message: Vec<u8> },
    SendMessage { peer: PeerId, message: Vec<u8> },
    GetWhitelisted { tx: oneshot::Sender<Vec<PeerId>> },
    AddWhitelisted { peer: PeerId },
    RemoveWhitelisted { peer: PeerId },
}

// Central structure of this application, that holds the swarm.
pub struct Network {
    // Libp2p swarm that manages all network interaction.
    swarm: Swarm<Behaviour>,
    // Topic that we are subscribing to.
    // Eventually this should be a list of topics.
    topic: IdentTopic,

    command_rx: mpsc::Receiver<Command>,
    inbound_message_tx: mpsc::Sender<(String, Vec<u8>, bool)>,
    event_tx: mpsc::Sender<NetworkEvent>,

    whitelisted: Vec<PeerId>,

    addresses: HashMap<PeerId, Vec<Multiaddr>>,
}

impl Network {
    // Create a new instance of `Network.`
    pub async fn new(
        keypair: identity::Keypair,
        command_rx: mpsc::Receiver<Command>,
        inbound_message_tx: mpsc::Sender<(String, Vec<u8>, bool)>,
        event_tx: mpsc::Sender<NetworkEvent>,
    ) -> Self {
        let local_peer_id = PeerId::from_public_key(&keypair.public());
        println!("[Network] Local PeerId: {}", local_peer_id);

        // Create a transport. The transport controls **how** we sent out data to the remote peer.
        let tcp_transport = tcp::TcpConfig::new();
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&keypair)
            .expect("Signing libp2p-noise static DH keypair failed.");
        let transport = tcp_transport
            .upgrade(core::upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(core::upgrade::SelectUpgrade::new(
                yamux::YamuxConfig::default(),
                mplex::MplexConfig::default(),
            ))
            .timeout(std::time::Duration::from_secs(20))
            .boxed();

        // Create a behaviour. The behaviour controls **what** we sent to the remote.
        // We use a custom behehaviour (see `Behaviour` docs).
        let behaviour = Behaviour::new(keypair).await.unwrap();

        // The swarm is libp2p single entry point that controls all network interaction.
        // It wraps the transport and the behaviour.
        let swarm = Swarm::new(transport, behaviour, local_peer_id);

        // Dummy topic for testing
        let topic = IdentTopic::new(TOPIC);

        // Return `Self`.
        Network {
            swarm,
            topic,
            inbound_message_tx,
            command_rx,
            event_tx,
            whitelisted: Vec::new(),
            addresses: HashMap::new(),
        }
    }

    // Start listening on the network on all interfaces (localhost, local network, etc.)
    pub fn start_listening(&mut self) {
        // Create an unspecified address (all zeroes).
        // This causes us to listen on all network interfaces on an OS-assigned address.
        let address: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse().unwrap();

        // Tell the swarm to start listening.
        self.swarm.listen_on(address).unwrap();
    }

    // Subscribe to our dummy topic.
    pub fn subscribe(&mut self) {
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&self.topic)
            .unwrap();
    }

    // Run an eternal loop that polls the swarm and for user input.
    //
    // The libp2p swarm is a state machine that needs to be polled continously
    // (e.g. via swarm.select_next_some()). If we don't poll it, nothing will happen.
    pub async fn run(mut self) {
        loop {
            // `Select` is a macro that simultaneously polls items.
            select! {
                // Poll the swarm for events.
                // Even if we would not care about the event, we have to poll the
                // swarm for it to make any progress.
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
                command = self.command_rx.select_next_some() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::PublishMessage { message } => self.publish_msg_to_swarm(&message),
            Command::SendMessage { peer, message }  => self.send_message(&peer, message),
            Command::GetWhitelisted { tx } => tx.send(self.whitelisted.clone()).unwrap(),
            Command::AddWhitelisted { peer } => {
                if !self.whitelisted.contains(&peer) {
                    self.whitelisted.push(peer);
                }

                // Maybe this is not so smart, when updating larger networks, since everyone would start
                // to connect a new peer all at once.
                self.dial_to_peer(peer).await;
            }
            Command::RemoveWhitelisted { peer } => {
                self.whitelisted.retain(|p| p != &peer);
            }
        }
    }

    // Publish the message in the gossipsub network
    fn publish_msg_to_swarm(&mut self, input: &[u8]) {
        match self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), input)
        {
            Ok(_) => {}
            Err(PublishError::InsufficientPeers) => {}
            Err(e) => {
                println!("[Network] Error sending to swarm: {:?}", e);
            }
        }
    }
    
    // Send a message to a single peer.
    fn send_message(&mut self, peer: &PeerId, data: Vec<u8>) {
        let _ = self.swarm.behaviour_mut().request_response.send_request(peer, data);
    }

    // Handle an event on our swarm.
    // This events could be about connections (e.g. connected/ disconnected to a peer), listeners
    // (e.g. new/ expired listening address) or events from our `Behaviour`.
    async fn handle_swarm_event<E>(&mut self, event: SwarmEvent<Event, E>) {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                if self.whitelisted.is_empty() || self.whitelisted.contains(&peer_id) {
                    println!("[Network] Connected to {:?}", peer_id);

                    self.event_tx
                        .send(NetworkEvent::ConnectionEstablished {
                            peer: peer_id.to_base58(),
                        })
                        .await
                        .unwrap();
                } else {
                    // TODO: reject connection without loosing the association from peer id to address
                    // (after disconnect_peer_id, connect(peer_id) fails with no_address)
                    println!(
                        "[Network] Disconnecting connection from not whitelisted peer {:?}",
                        peer_id
                    );
                    let _ = self.swarm.disconnect_peer_id(peer_id);

                    self.event_tx
                        .send(NetworkEvent::ConnectionRejected {
                            peer: peer_id.to_base58(),
                        })
                        .await
                        .unwrap();
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("[Network] Listening on {:?}", address);
                self.event_tx
                    .send(NetworkEvent::NewListenAddress {
                        addr: address.to_string().split("/").nth(2).unwrap().into(),
                    })
                    .await
                    .unwrap();
            }
            // Event issued my the Mdns protocol behaviour.
            SwarmEvent::Behaviour(Event::Mdns(ev)) => {
                self.handle_mdns_event(ev).await;
            }
            // Event issued my the Gossisub protocol behaviour.
            SwarmEvent::Behaviour(Event::Gossipsub(ev)) => {
                self.handle_gossisub_event(ev).await;
            }
            // Event issued my the Request Response protocol behaviour.
            SwarmEvent::Behaviour(Event::ReqRes(ev)) => {
                self.handle_req_res_event(ev).await;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("[Network] Connection to {:?} closed.", peer_id);
                self.event_tx
                    .send(NetworkEvent::ConnectionClosed {
                        peer: peer_id.to_base58(),
                    })
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }

    // Handle event created by our inner MDNS behaviour.
    async fn handle_mdns_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(discovered) = event {
            // We discovered new peers in the local network.
            // Dial each peer. Once the connection is established the gossibsub protocol will make
            // the peers exchange what topics they are subscribed to.
            // When publishing to a topic, we thus then know whom to send the message to.
            let mut distinct_peers = HashSet::new();
            for (peer, addr) in discovered {
                distinct_peers.insert(peer);
                let addrs = self.addresses.entry(peer).or_default();
                addrs.push(addr);
            }
            for peer in distinct_peers {
                self.event_tx
                    .send(NetworkEvent::PeerDiscovered {
                        peer: peer.to_base58(),
                    })
                    .await
                    .unwrap();

                if self.whitelisted.contains(&peer) {
                    println!("[Network] Connecting to whitelisted peer {:?}", peer);
                    self.dial_to_peer(peer).await;
                } else {
                    println!("[Network] Got peer not whitelisted {:?}", peer);
                }
            }
        } else if let MdnsEvent::Expired(expired) = event {
            for (peer, _) in expired {
                self.event_tx
                    .send(NetworkEvent::PeerExpired {
                        peer: peer.to_base58(),
                    })
                    .await
                    .unwrap();
            }
        }
    }

    async fn dial_to_peer(&mut self, peer: PeerId) {
        if !self.addresses.contains_key(&peer) {
            println!("[Network] Could not find addresses for {:?}", peer);
            return;
        }

        let opts = DialOpts::peer_id(peer)
            .addresses(self.addresses.get(&peer).unwrap().clone())
            .build();
        match self.swarm.dial(opts) {
            Ok(_) => {}
            Err(DialError::DialPeerConditionFalse(_)) => {}
            Err(e) => {
                println!("[Network] Got error connecting to {:?}: {:?}", peer, e);
            }
        }
    }

    // Handle event created by our inner GossibSub behaviour.
    async fn handle_gossisub_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            message:
                GossipsubMessage {
                    data,
                    source: Some(source),
                    ..
                },
            ..
        } = event
        {
            self.inbound_message_tx
                .send((source.to_base58(), data, true))
                .await
                .unwrap();
        }
    }

    // Handle event created by our inner Request Response behaviour.
    async fn handle_req_res_event(&mut self, event: RequestResponseEvent<Vec<u8>, Ack>) {
        if let RequestResponseEvent::Message { peer, message:
        RequestResponseMessage::Request { request_id: _, request, channel } } = event {
            let _ = self.swarm.behaviour_mut().request_response.send_response(channel, Ack);
            self.inbound_message_tx.send((peer.to_base58(), request, false)).await.unwrap();
        }
    }
}

// Custom `NetworkBehaviour`.
//
// A network behaviour in libp2p consists of one or multiple
// protocols that control what data is sent to the remote peer.
//
// These protocol can run automatically in the background (like mdns,
// which periodically queries for peers in the same local network) or
// on demand (like sending a gossibsub message).
//
// We use the `#[derive(NetworkBehaviour)]` annotation to wrap two existing
// behaviours into our own custom one.
// Events from a network behaviour are returned when polling the swarm
// via `SwarmEvent::Behaviour`. Because the two wrapped behaviours return
// different events (GossipSubEvent / MdnsEvent), we create an enum `Event`
// (see below) that wraps the two possible events.
// With `#[behaviour(out_event = "Event")]` we specify that our own `Event`
// should be returned by our Behaviour.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Event")]
struct Behaviour {
    // Gossisub PubSub protocol.
    // Allows publishing messages in a network to a cerain topic.
    gossipsub: Gossipsub,
    // Request Response protocol.
    // Allows direct message to remote peers.
    request_response: RequestResponse<Codec>,
    // Multicast DNS protocol for peer discovery in the local network.
    mdns: Mdns,
}

impl Behaviour {
    // Create a new instance of a `Behaviour`.
    async fn new(keypair: identity::Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(keypair),
            GossipsubConfig::default(),
        )
        .unwrap();
        let cfg = RequestResponseConfig::default();
        let request_response = RequestResponse::new(Codec, iter::once((Protocol{}, ProtocolSupport::Full)), cfg);
        let mdns = Mdns::new(MdnsConfig::default()).await.unwrap();
        let behaviour = Behaviour { gossipsub, mdns, request_response };
        Ok(behaviour)
    }
}

// Custom event that wraps the events from out inner behaviours.
#[derive(Debug)]
enum Event {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
    ReqRes(RequestResponseEvent<Vec<u8>, Ack>)
}

impl From<MdnsEvent> for Event {
    fn from(ev: MdnsEvent) -> Self {
        Event::Mdns(ev)
    }
}

impl From<GossipsubEvent> for Event {
    fn from(ev: GossipsubEvent) -> Self {
        Event::Gossipsub(ev)
    }
}

impl From<RequestResponseEvent<Vec<u8>, Ack>> for Event {
    fn from(ev: RequestResponseEvent<Vec<u8>, Ack>) -> Self {
        Event::ReqRes(ev)
    }
}
