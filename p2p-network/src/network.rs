use futures::{channel::mpsc, select, SinkExt, StreamExt};
use libp2p::{
    gossipsub::{
        Gossipsub, GossipsubConfig, GossipsubEvent, GossipsubMessage, IdentTopic,
        MessageAuthenticity,
    },
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    Multiaddr, NetworkBehaviour, PeerId, Swarm,
};
use std::collections::HashSet;

// Fixed topic for the first PoC.
const TOPIC: &str = "topic";

// Central structure of this application, that holds the swarm.
pub struct Network {
    // Libp2p swarm that manages all network interaction.
    swarm: Swarm<Behaviour>,
    // Topic that we are subscribing to.
    // Eventually this should be a list of topics.
    topic: IdentTopic,

    send_message_rx: mpsc::Receiver<Vec<u8>>,
    inbound_message_tx: mpsc::Sender<Vec<u8>>,
}

impl Network {
    // Create a new instance of `Network.`
    pub async fn new(
        send_message_rx: mpsc::Receiver<Vec<u8>>,
        inbound_message_tx: mpsc::Sender<Vec<u8>>,
    ) -> Self {
        // Authentication keypair.
        // Used to derive a unique PeerId and the keypair for encryption on the
        // Transport layer with the Noise protocol (https://noiseprotocol.org/noise.html).
        let keypair = identity::Keypair::generate_ed25519();

        let local_peer_id = PeerId::from_public_key(&keypair.public());
        println!("Local PeerId: {}", local_peer_id);

        // Create a transport. The transport controls **how** we sent out data to the remote peer.
        //
        // We use an existing transport from libp2p that can be used for testing/
        // Supports TCP, websockets and DNS name resolution
        let transport = libp2p::development_transport(keypair.clone())
            .await
            .unwrap();

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
            send_message_rx,
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
                message = self.send_message_rx.select_next_some() => {
                    self.send_msg_to_swam(&message);
                }
            }
        }
    }

    // Handle input from the user.
    // This publishes the input as a message in the gossibsub network
    fn send_msg_to_swam(&mut self, input: &[u8]) {
        match self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), input)
        {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    // Handle an event on our swarm.
    // This events could be about connections (e.g. connected/ disconnected to a peer), listeners
    // (e.g. new/ expired listening address) or events from our `Behaviour`.
    async fn handle_swarm_event<E>(&mut self, event: SwarmEvent<Event, E>) {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {:?}", peer_id);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            // Event issued my the Mdns protocol behaviour.
            SwarmEvent::Behaviour(Event::Mdns(ev)) => {
                self.handle_mdns_event(ev);
            }
            // Event issued my the Gossibsub protocol behaviour.
            SwarmEvent::Behaviour(Event::Gossipsub(ev)) => {
                self.handle_gossisub_event(ev).await;
            }
            _ => {}
        }
    }

    // Handle event created by our inner MDNS behaviour.
    fn handle_mdns_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(discovered) = event {
            let peers: HashSet<PeerId> = discovered.map(|(peer, _addr)| peer).collect();
            // We discovered new peers in the local network.
            // Dial each peer. Once the connection is established the gossibsub protocol will make
            // the peers exchange what topics they are subscribed to.
            // When publishing to a topic, we thus then know whom to send the message to.
            for peer in peers {
                self.swarm.dial(peer).unwrap();
            }
        }
    }

    // Handle event created by our inner GossibSub behaviour.
    async fn handle_gossisub_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            message: GossipsubMessage { data, .. },
            ..
        } = event
        {
            // We received a message that was published by a remote peer to a topic
            // we are subscribing to.
            self.inbound_message_tx.send(data).await.unwrap();
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
        let mdns = Mdns::new(MdnsConfig::default()).await.unwrap();
        let behaviour = Behaviour { gossipsub, mdns };
        Ok(behaviour)
    }
}

// Custom event that wraps the events from out inner behaviours.
#[derive(Debug)]
enum Event {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
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
