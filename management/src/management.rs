use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time;

use crate::upgrade;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::select;
use futures::StreamExt;
use message::control_message::MessageType;
use message::ControlMessage;
use p2p_network::NetworkComponent;
use p2p_network::NetworkEvent;
use prost::bytes::Bytes;
use prost::Message;
use upgrade::UpgradeServer;

pub const CURRENT_VERSION: Option<&str> = option_env!("DF_VERSION");

#[derive(Debug)]
pub enum UserCommand {
    SendMsg { peer: String, message: String },
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
mod message {
    include!(concat!(env!("OUT_DIR"), "/management.control_message.rs"));
}

impl ControlMessage {
    pub fn new<T: Into<String>, S: Into<String>>(
        message_type: MessageType,
        payload: T,
        receiver: S,
    ) -> Self {
        ControlMessage {
            message_type: message_type as i32,
            receiver: receiver.into(),
            sender: "".into(),
            payload: payload.into(),
        }
    }
}

pub struct Management {
    recv_msg_rx: mpsc::Receiver<(String, Vec<u8>)>,
    user_input_rx: mpsc::Receiver<UserCommand>,
    event_rx: mpsc::Receiver<NetworkEvent>,

    autorized_senders: Vec<String>,
    aliases: HashMap<String, String>,
    alias: String,

    network: NetworkComponent,
    upgrader: UpgradeServer,

    discovered_peers: Vec<String>,
    rejected_peers: Vec<String>,
    connected_peers: Vec<String>,
    listening_addrs: Vec<String>,

    upgrade_in_progress: bool,
}

impl Management {
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
        let network = NetworkComponent::init(private_key, recv_msg_tx, network_event_tx);

        Management {
            recv_msg_rx,
            user_input_rx,
            network,
            event_rx: network_event_rx,
            autorized_senders: vec![],
            aliases: HashMap::new(),
            alias: "".into(),
            upgrader: UpgradeServer::new(),
            discovered_peers: vec![],
            rejected_peers: vec![],
            connected_peers: vec![],
            listening_addrs: vec![],
            upgrade_in_progress: false,
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
                input = self.user_input_rx.select_next_some() => {
                    self.handle_user_command(input).await;
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

                if self.alias != "" {
                    self.send(ControlMessage::new(
                        MessageType::PublishAlias,
                        self.alias.clone(),
                        peer.clone(),
                    ))
                    .await;
                }

                if let Some(current_version) = CURRENT_VERSION {
                    self.send(ControlMessage::new(
                        MessageType::NetworkBinaryVersion,
                        current_version,
                        peer.clone(),
                    ))
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
                self.send(ControlMessage::new(
                    MessageType::DisplayMessage,
                    message,
                    peer,
                ))
                .await;
            }
            UserCommand::Whitelist(new_peer) => {
                let ctrl = ControlMessage::new(MessageType::AddWhitelistPeer, new_peer.clone(), "");
                self._handle_message(ctrl.clone()).await;

                // notify the old peers of the new peer
                thread::sleep(time::Duration::from_millis(200));
                self.send(ctrl).await;
                thread::sleep(time::Duration::from_millis(200));

                // Notify the new peer of the old whitelist
                let list = self.network.get_whitelisted().await;
                for peer in list {
                    self.send(ControlMessage::new(
                        MessageType::AddWhitelistPeer,
                        peer,
                        new_peer.clone(),
                    ))
                    .await;
                }
            }
            UserCommand::Authorize(peer) => {
                let ctrl = ControlMessage::new(MessageType::AddWhitelistSender, peer, "");
                self._handle_message(ctrl.clone()).await;
                self.send(ctrl).await;
            }
            UserCommand::Alias(alias) => {
                self.send(ControlMessage::new(
                    MessageType::PublishAlias,
                    alias.clone(),
                    "",
                ))
                .await;
                self.alias = alias.into();
            }
            UserCommand::UpgradeSelf(network_addr) => {
                let _ = UpgradeServer::upgrade_binary(network_addr.into());
            }
            UserCommand::Upgrade(a, b) => {
                self.send(ControlMessage::new(MessageType::Upgrade, b, a))
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

    // Handle an incoming message as as base64-encoded string (for testing).
    pub async fn receive(&mut self, sender: String, msg: String) {
        let bytes = base64::decode(msg).unwrap();
        self.network_receive(sender, &bytes).await;
    }

    // Receive data from the network.
    pub async fn network_receive(&mut self, _sender: String, data: &[u8]) {
        let bytes = std::boxed::Box::from(data);
        let decoded = ControlMessage::decode(Bytes::from(bytes)).unwrap();
        self._handle_message(decoded).await;
    }

    // Send a ControlMessage as a base64 encoded string to the network layer.
    //
    // The sender id will automatically be set.
    pub async fn send(&mut self, msg: ControlMessage) {
        let message = ControlMessage {
            sender: self.network.local_peer_id(),
            receiver: self._resolve_alias(msg.receiver),
            ..msg
        };
        let encoded = message.encode_to_vec();
        println!(
            "[Management] Sending message of type {:?} to {:?}",
            MessageType::from_i32(message.message_type).unwrap(),
            message.receiver.get(44..).unwrap_or("broadcast")
        );

        self.network.send_message(encoded.to_vec()).await;
    }

    // Return the alias id resolves to or id itself
    fn _resolve_alias(&mut self, id: String) -> String {
        return self.aliases.get(&id).unwrap_or(&id).clone();
    }

    async fn _handle_message(&mut self, msg: ControlMessage) {
        println!(
            "[Management] Got message of type {:?} from {:?}",
            MessageType::from_i32(msg.message_type).unwrap(),
            &msg.sender.get(44..).unwrap_or("broadcast"),
        );

        // return if the message is not broadcast and not for me
        if !msg.receiver.is_empty() && msg.receiver != self.network.local_peer_id() {
            println!("[Management] Ignoring message for other peer");
            return;
        }

        // return if there are authorized senders and the message sender is not one of them
        if !self.autorized_senders.is_empty() && !self.autorized_senders.contains(&msg.sender) {
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
                self.autorized_senders.push(msg.payload);
            }
            Some(MessageType::PublishAlias) => {
                if self.aliases.contains_key(&msg.payload) {
                    println!(
                        "[Management] Rejected new alias {:?} for {:?}",
                        &msg.payload,
                        &msg.sender.get(44..).unwrap_or("broadcast")
                    );
                    return;
                }

                println!(
                    "[Management] Got new alias {:?} for {:?}",
                    &msg.payload,
                    &msg.sender.get(44..).unwrap_or("broadcast"),
                );

                // remove previous alias for sender
                let prev_alias = self._resolve_alias(msg.sender.clone());
                let _ = self.aliases.remove(&prev_alias);

                // add new alias for sender
                self.aliases.insert(msg.payload, msg.sender.clone());
            }
            Some(MessageType::NetworkSolicitation) => {
                // Send current alias if there is one
                if self.alias != "" {
                    self.send(ControlMessage {
                        message_type: MessageType::PublishAlias as i32,
                        sender: "".into(),
                        receiver: msg.sender,
                        payload: self.alias.clone(),
                    })
                    .await;
                }
            }
            Some(MessageType::Upgrade) => {
                println!("[Management] Got upgrade from {}", msg.sender);
                let _ = UpgradeServer::upgrade_binary(msg.payload);
            }
            Some(MessageType::RequestUpgrade) => {
                println!("[Management] Got upgrade request from {}", &msg.sender);
                if self.upgrade_in_progress {
                    return;
                }

                self.upgrader.serve_binary_once().await;

                for addr in self.listening_addrs.clone().iter() {
                    let mut a = addr.clone();
                    a.push_str(":");
                    a.push_str(upgrade::UPGRADE_SERVER_PORT);

                    self.send(ControlMessage::new(MessageType::Upgrade, a, &msg.sender))
                        .await;
                }
            }
            Some(MessageType::NetworkBinaryVersion) => {
                println!("[Management] Got binary version from {}", &msg.sender);
                if CURRENT_VERSION.is_some()
                    && String::from(CURRENT_VERSION.unwrap()).ge(&msg.payload)
                {
                    return;
                }
                if self.upgrade_in_progress {
                    return;
                }
                self.upgrade_in_progress = true;

                self.send(ControlMessage::new(
                    MessageType::RequestUpgrade,
                    "",
                    &msg.sender,
                ))
                .await;
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
