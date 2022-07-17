use crate::dht::Dht;
use crate::protocol::{Alias, ControlMessage, MessageType, NetworkState};
use crate::upgrade;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::select;
use futures::StreamExt;
use p2p_network::NetworkEvent;
use p2p_network::NetworkLayer;
use prost::bytes::Bytes;
use prost::Message;
use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time;
use upgrade::UpgradeServer;

pub const CURRENT_VERSION: Option<&str> = option_env!("DF_VERSION");

#[derive(Debug)]
pub enum UserCommand {
    SendMsg {
        peer: Option<String>,
        message: String,
    },
    Whitelist(String),
    Authorize(String),
    Alias(String),
    UpgradeSelf(String),
    Upgrade(String, String),
    Serve(String),
    ServeStop,
    GetPeerId(oneshot::Sender<String>),
    GetAlias(oneshot::Sender<String>),
    GetAliases(oneshot::Sender<HashMap<String, String>>),
    GetDiscovered(oneshot::Sender<Vec<String>>),
    GetConnected(oneshot::Sender<Vec<String>>),
    GetRejected(oneshot::Sender<Vec<String>>),
}

#[cfg(feature = "display")]
#[link(name = "display")]
extern "C" {
    pub fn toDisplay(message: *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}

pub struct Management<T> {
    recv_msg_rx: mpsc::Receiver<(String, Vec<u8>)>,
    user_input_rx: mpsc::Receiver<UserCommand>,
    event_rx: mpsc::Receiver<NetworkEvent>,

    authorized_senders: Vec<String>,
    aliases: HashMap<String, String>,
    alias: String,

    network: T,
    upgrader: UpgradeServer,

    discovered_peers: Vec<String>,
    rejected_peers: Vec<String>,
    connected_peers: Vec<String>,
    listening_addrs: Vec<String>,

    upgrade_in_progress: bool,

    local_id: String,

    dht: Dht,
}

impl<T: NetworkLayer> Management<T> {
    pub fn new(user_input_rx: mpsc::Receiver<UserCommand>) -> Self {
        // it appears there is a deadlock in here somewhere... so we need some buffer to clear it.
        let (recv_msg_tx, recv_msg_rx) = mpsc::channel(10);
        let (network_event_tx, network_event_rx) = mpsc::channel(10);

        let mut private_key: Option<&Path> = None;
        let mut pk: String;

        let mut iter = std::env::args().into_iter();
        loop {
            let arg = iter.next();

            match arg {
                None => {
                    break;
                }
                Some(arg) => {
                    if arg == "--private-key" {
                        let n = iter.next();
                        if n.is_some() {
                            pk = n.unwrap();
                            private_key = Some(Path::new(&pk));
                        }
                    }
                }
            }
        }
        let network = T::init(private_key, recv_msg_tx, network_event_tx);
        let local_id = network.local_peer_id();

        Management {
            recv_msg_rx,
            user_input_rx,
            network,
            event_rx: network_event_rx,
            authorized_senders: Vec::new(),
            aliases: HashMap::new(),
            alias: String::new(),
            upgrader: UpgradeServer::new(),
            discovered_peers: Vec::new(),
            rejected_peers: Vec::new(),
            connected_peers: Vec::new(),
            listening_addrs: Vec::new(),
            upgrade_in_progress: false,
            local_id: local_id.clone(),
            dht: Dht::new(local_id),
        }
    }

    pub async fn run(mut self) {
        write_to_display("Initializing".into());
        loop {
            // `Select` is a macro that simultaneously polls items.
            select! {
                // Poll the swarm for events.
                // Even if we would not care about the event, we have to poll the
                // swarm for it to make any progress.
                (sender, message) = self.recv_msg_rx.select_next_some() => {
                    self.network_receive(sender, &message).await;
                }
                // Poll for user input.
                input = self.user_input_rx.next() => {
                    match input {
                        Some(input) => self.handle_user_command(input).await,
                        None => {
                            self.shutdown().await;
                            return
                        }
                    }
                }
                event = self.event_rx.select_next_some() => {
                    self.handle_network_event(event).await;
                }
            }
        }
    }

    pub async fn handle_network_event(&mut self, event: NetworkEvent) {
        match event {
            NetworkEvent::PeerDiscovered { peer } => {
                if !self.discovered_peers.contains(&peer) {
                    self.discovered_peers.push(peer);
                }
            }
            NetworkEvent::ConnectionEstablished { peer } => {
                // wait a bit for all connections to be established
                thread::sleep(time::Duration::from_millis(500));

                if self.connected_peers.is_empty() {
                    // Connected to first peer in the network.
                    self.send(
                        ControlMessage::new(MessageType::PeerConnected, self.local_id.clone()),
                        None,
                    )
                    .await;
                    // Fetch network state
                    self.send(
                        ControlMessage::new(MessageType::NetworkSolicitation, ""),
                        Some(peer.clone()),
                    )
                    .await;
                }

                if self.alias != "" {
                    self.send(
                        ControlMessage::new(MessageType::PublishAlias, self.alias.clone()),
                        Some(peer.clone()),
                    )
                    .await;
                }

                if let Some(current_version) = CURRENT_VERSION {
                    self.send(
                        ControlMessage::new(MessageType::NetworkBinaryVersion, current_version),
                        Some(peer.clone()),
                    )
                    .await;
                }

                self.rejected_peers.retain(|p| p != &peer);
                if !self.connected_peers.contains(&peer) {
                    self.connected_peers.push(peer);
                }
            }
            NetworkEvent::ConnectionClosed { peer } => {
                self.connected_peers.retain(|p| p != &peer);
                self.rejected_peers.retain(|p| p != &peer);
                self.discovered_peers.retain(|p| p != &peer);
            }
            NetworkEvent::ConnectionRejected { peer } => {
                if !self.rejected_peers.contains(&peer) {
                    self.rejected_peers.push(peer);
                }
            }
            NetworkEvent::PeerExpired { peer } => {
                self.connected_peers.retain(|p| p != &peer);
                self.rejected_peers.retain(|p| p != &peer);
                self.discovered_peers.retain(|p| p != &peer);
            }
            NetworkEvent::NewListenAddress { addr } => {
                if !self.listening_addrs.contains(&addr) {
                    self.listening_addrs.push(addr);
                }
            }
        }
    }

    pub async fn handle_user_command(&mut self, command: UserCommand) {
        match command {
            UserCommand::SendMsg { peer, message } => {
                let peer = match peer {
                    Some(peer) => peer,
                    None => {
                        // Publish message to whole network.
                        self.dht.store_broadcast_content(message.clone());
                        self.send(
                            ControlMessage::new(MessageType::DisplayMessage, message),
                            None,
                        )
                        .await;
                        return;
                    }
                };
                for closest in self.dht.get_closest_peers(&peer) {
                    if closest == peer {
                        self.send(
                            ControlMessage::new(MessageType::DisplayMessage, message.clone()),
                            Some(peer.clone()),
                        )
                        .await;
                    } else if closest == self.local_id {
                        println!("[Management] Storing message for {:?}", peer);
                        self.dht.store(peer.clone(), message.clone());
                    } else {
                        self.send(
                            ControlMessage::new(MessageType::StoreMessage, message.clone()),
                            Some(peer.clone()),
                        )
                        .await;
                    }
                }
            }
            UserCommand::Whitelist(new_peer) => {
                let whitelist = self.network.get_whitelisted().await;
                if whitelist.contains(&new_peer) {
                    return;
                }
                self.whitelist_peer(new_peer).await;
            }
            UserCommand::Authorize(peer) => {
                let ctrl = ControlMessage::new(MessageType::AddWhitelistSender, peer);
                self._handle_message(self.local_id.clone(), ctrl.clone())
                    .await;
                self.send(ctrl, None).await;
            }
            UserCommand::Alias(alias) => {
                self.send(
                    ControlMessage::new(MessageType::PublishAlias, alias.clone()),
                    None,
                )
                .await;
                self.alias = alias.into();
            }
            UserCommand::UpgradeSelf(network_addr) => {
                let _ = UpgradeServer::upgrade_binary(network_addr.into());
            }
            UserCommand::Upgrade(a, b) => {
                let target = (!a.is_empty()).then(|| a);
                self.send(ControlMessage::new(MessageType::Upgrade, b), target)
                    .await;
            }
            UserCommand::Serve(file_path) => {
                self.upgrader.serve(file_path.into()).await;
            }
            UserCommand::ServeStop => {
                self.upgrader.stop_serving().await;
            }
            UserCommand::GetPeerId(tx) => {
                tx.send(self.network.local_peer_id()).unwrap();
            }
            UserCommand::GetAlias(tx) => {
                tx.send(self.alias.clone()).unwrap();
            }
            UserCommand::GetAliases(tx) => {
                tx.send(self.aliases.clone()).unwrap();
            }
            UserCommand::GetDiscovered(tx) => {
                tx.send(self.discovered_peers.clone()).unwrap();
            }
            UserCommand::GetConnected(tx) => {
                tx.send(self.connected_peers.clone()).unwrap();
            }
            UserCommand::GetRejected(tx) => {
                tx.send(self.rejected_peers.clone()).unwrap();
            }
        }
    }

    pub async fn shutdown(mut self) {
        self.send(
            ControlMessage::new(MessageType::PeerDisconnected, self.local_id.clone()),
            None,
        )
        .await;
        thread::sleep(time::Duration::from_millis(500));
    }

    pub async fn whitelist_peer(&mut self, new_peer: String) {
        let ctrl = ControlMessage::new(MessageType::AddWhitelistPeer, new_peer.clone());
        self._handle_message(self.local_id.clone(), ctrl.clone())
            .await;

        // notify the old peers of the new peer
        thread::sleep(time::Duration::from_millis(200));
        self.send(ctrl, None).await;
        thread::sleep(time::Duration::from_millis(200));
        self._handle_message(
            new_peer.clone(),
            ControlMessage::new(MessageType::NetworkSolicitation, ""),
        )
        .await;
    }

    // Handle an incoming message as as base64-encoded string (for testing).
    pub async fn receive(&mut self, sender: String, msg: String) {
        let bytes = base64::decode(msg).unwrap();
        self.network_receive(sender, &bytes).await;
    }

    // Receive data from the network.
    pub async fn network_receive(&mut self, sender: String, data: &[u8]) {
        let bytes = std::boxed::Box::from(data);
        let decoded = ControlMessage::decode(Bytes::from(bytes)).unwrap();
        self._handle_message(sender, decoded).await;
    }

    // Send a ControlMessage as a base64 encoded string to the network layer.
    //
    // The sender id will automatically be set.
    pub async fn send(&mut self, msg: ControlMessage, target: Option<String>) {
        let encoded = msg.encode_to_vec();
        println!(
            "[Management] Sending message of type {:?} to {:?}",
            MessageType::from_i32(msg.message_type).unwrap(),
            target.clone().unwrap_or("broadcast".into())
        );

        match target {
            Some(t) => self.network.send_message(t, encoded.to_vec()).await,
            None => self.network.publish_message(encoded.to_vec()).await,
        }
    }

    // Return the alias id resolves to or id itself
    fn _resolve_alias(&mut self, id: String) -> String {
        return self.aliases.get(&id).unwrap_or(&id).clone();
    }

    async fn _handle_message(&mut self, sender: String, msg: ControlMessage) {
        println!(
            "[Management] Got message of type {:?} from {:?}",
            MessageType::from_i32(msg.message_type).unwrap(),
            &sender,
        );

        // return if there are authorized senders and the message sender is not one of them
        if !self.authorized_senders.is_empty() && !self.authorized_senders.contains(&sender) {
            println!("[Management] Unauthorized sender: {:?}", msg);
            return;
        }

        match MessageType::from_i32(msg.message_type) {
            Some(MessageType::DisplayMessage) => {
                write_to_display(msg.payload);
            }
            Some(MessageType::AddWhitelistPeer) => {
                println!("[Management] Whitelisting peer: {:?}", &msg.payload);
                self.network.add_whitelisted(msg.payload).await;
            }
            Some(MessageType::AddWhitelistSender) => {
                println!("[Management] Authorizing sender: {:?}", &msg.payload);
                self.authorized_senders.push(msg.payload);
            }
            Some(MessageType::PublishAlias) => {
                if self.aliases.contains_key(&msg.payload) {
                    println!(
                        "[Management] Rejected new alias {:?} for {:?}",
                        &msg.payload, sender
                    );
                    return;
                }

                println!(
                    "[Management] Got new alias {:?} for {:?}",
                    &msg.payload, sender,
                );

                // remove previous alias for sender
                let prev_alias = self._resolve_alias(sender.clone());
                let _ = self.aliases.remove(&prev_alias);

                // add new alias for sender
                self.aliases.insert(msg.payload, sender);
            }
            Some(MessageType::NetworkSolicitation) => {
                let aliases = self
                    .aliases
                    .clone()
                    .into_iter()
                    .map(|(peer, alias)| Alias { peer, alias })
                    .collect();
                let connected = self.dht.get_peers();
                let whitelisted = self.network.get_whitelisted().await;
                let whitelisted_sender = self.authorized_senders.clone();
                self.send(
                    ControlMessage {
                        message_type: MessageType::State as i32,
                        state: Some(NetworkState {
                            whitelisted,
                            connected,
                            whitelisted_sender,
                            aliases,
                        }),
                        payload: String::new(),
                    },
                    Some(sender),
                )
                .await;
            }
            Some(MessageType::Upgrade) => {
                println!("[Management] Got upgrade from {}", sender);
                let _ = UpgradeServer::upgrade_binary(msg.payload);
            }
            Some(MessageType::RequestUpgrade) => {
                println!("[Management] Got upgrade request from {}", sender);
                if self.upgrade_in_progress {
                    return;
                }

                self.upgrader.serve_binary_once().await;

                for addr in self.listening_addrs.clone().iter() {
                    let mut a = addr.clone();
                    a.push_str(":");
                    a.push_str(upgrade::UPGRADE_SERVER_PORT);

                    self.send(
                        ControlMessage::new(MessageType::Upgrade, a),
                        Some(sender.clone()),
                    )
                    .await;
                }
            }
            Some(MessageType::NetworkBinaryVersion) => {
                println!("[Management] Got binary version from {}", sender);
                if CURRENT_VERSION.is_some()
                    && String::from(CURRENT_VERSION.unwrap()).ge(&msg.payload)
                {
                    return;
                }
                if self.upgrade_in_progress {
                    return;
                }
                self.upgrade_in_progress = true;

                self.send(
                    ControlMessage::new(MessageType::RequestUpgrade, ""),
                    Some(sender),
                )
                .await;
            }
            Some(MessageType::PeerConnected) => self.dht.add_peer(msg.payload),
            Some(MessageType::PeerDisconnected) => self.dht.remove_peer(&msg.payload),
            Some(MessageType::RequestMessage) => {
                if let Some(message) = self.dht.get_content(&sender) {
                    self.send(
                        ControlMessage::new(MessageType::DisplayMessage, message),
                        Some(sender),
                    )
                    .await;
                }
            }
            Some(MessageType::StoreMessage) => self.dht.store(sender, msg.payload),
            Some(MessageType::State) => {
                println!(
                    "[Management] Got network state from {}: {:?}",
                    sender, msg.state
                );
                let state = msg.state.unwrap();
                for peer in state.connected {
                    self.dht.add_peer(peer)
                }
                for peer in state.whitelisted {
                    self.network.add_whitelisted(peer).await;
                }
                for peer in state.whitelisted_sender {
                    self.authorized_senders.push(peer)
                }
                for Alias { peer, alias } in state.aliases {
                    self.aliases.insert(peer, alias);
                }
                if let Some(other) = self
                    .dht
                    .get_closest_other(&self.local_id)
                    .into_iter()
                    .next()
                {
                    self.send(
                        ControlMessage::new(MessageType::RequestMessage, ""),
                        Some(other),
                    )
                    .await;
                }
            }
            None => {
                println!("Could not parse message");
            }
        }
    }
}

#[cfg(feature = "display")]
fn write_to_display(mut data: String) {
    println!("[DISPLAY] Sending data to display: {:?}", data);
    unsafe {
        data = data.replace(|c: char| !c.is_ascii(), "");
        data.push('\0');
        toDisplay(data.as_mut_ptr().cast());
    }
}

#[cfg(not(feature = "display"))]
fn write_to_display(data: String) {
    println!("[DISPLAY] MOCK sending data to display: {:?}", data);
}
