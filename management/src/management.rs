use std::collections::HashMap;
use std::path::Path;
use std::thread;
use std::time;

mod upgrade;

use crate::message::control_message::MessageType;
use crate::message::ControlMessage;
use async_std::io;
use futures::channel::mpsc;
use futures::select;
use futures::AsyncBufReadExt;
use futures::StreamExt;
use p2p_network::NetworkComponent;
use p2p_network::NetworkEvent;
use p2p_network::NetworkLayer;
use prost::bytes::Bytes;
use prost::Message;
use upgrade::UpgradeServer;

pub const CURRENT_VERSION: Option<&str> = option_env!("DF_VERSION");

#[cfg(feature = "display")]
#[link(name = "display")]
extern "C" {
    pub fn toDisplay(message: *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
pub mod message {
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

pub struct Management<T> {
    display_show: fn(data: String),
    recv_msg_rx: mpsc::Receiver<(String, Vec<u8>)>,
    event_rx: mpsc::Receiver<NetworkEvent>,

    autorized_senders: Vec<String>,
    aliases: HashMap<String, String>,
    alias: String,

    network: T,
    upgrader: UpgradeServer,

    discovered_peers: Vec<String>,
    rejected_peers: Vec<String>,
    connected_peers: Vec<String>,
    listening_addrs: Vec<String>,

    upgrade_in_progress: bool,
    table_mode: bool,
    table_receiver: String,
    table: String,
}

impl<T: NetworkLayer> Management<T> {
    pub fn new(display_show: fn(data: String)) -> Self {
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

        Management {
            display_show,
            recv_msg_rx,
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
            table_mode: false,
            table_receiver: String::new(),
            table: String::new(),
        }
    }

    pub async fn run(mut self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        (self.display_show)("Initializing".into());
        loop {
            // `Select` is a macro that simultaneously polls items.
            select! {
                // Poll the swarm for events.
                // Even if we would not care about the event, we have to poll the
                // swarm for it to make any progress.
                (sender, message) = self.recv_msg_rx.select_next_some() => {
                    self.network_receive(sender, &message).await;
                }
                // Poll for user input from stdin.
                line = stdin.select_next_some() => {
                    let input = line.expect("Stdin not to close");
                    self.handle_user_input(input).await;
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

    pub async fn handle_user_input(&mut self, msg: String) {
        if self.table_mode {
            if msg.is_empty() {
                self.table_mode = false;
                self.send(ControlMessage::new(
                    MessageType::DisplayMessage,
                    &self.table,
                    &self.table_receiver,
                ))
                .await;

                self.table_mode = false;
                return;
            }
            self.table.push_str(&msg);
            self.table.push_str("\n");
            return;
        }

        if let Some(msg) = msg.strip_prefix("send ") {
            self.send(ControlMessage::new(MessageType::DisplayMessage, msg, ""))
                .await;
        } else if let Some(msg) = msg.strip_prefix("sendto ") {
            let parts = msg.split_once(" ").unwrap();
            self.send(ControlMessage::new(
                MessageType::DisplayMessage,
                parts.1,
                parts.0,
            ))
            .await;
        } else if let Some(msg) = msg.strip_prefix("whitelist ") {
            let new_peer = msg;

            let ctrl = ControlMessage::new(MessageType::AddWhitelistPeer, new_peer, "");
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
                    new_peer,
                ))
                .await;
            }
        } else if let Some(msg) = msg.strip_prefix("authorize ") {
            let ctrl = ControlMessage::new(MessageType::AddWhitelistSender, msg, "");

            self._handle_message(ctrl.clone()).await;
            self.send(ctrl).await;
        } else if let Some(msg) = msg.strip_prefix("alias ") {
            self.send(ControlMessage::new(MessageType::PublishAlias, msg, ""))
                .await;
            self.alias = msg.into();
        } else if let Some(msg) = msg.strip_prefix("upgrade self ") {
            let _ = UpgradeServer::upgrade_binary(msg.into());
        } else if let Some(msg) = msg.strip_prefix("upgrade ") {
            let parts = msg.split_once(" ").unwrap();
            self.send(ControlMessage::new(MessageType::Upgrade, parts.1, parts.0))
                .await;
        } else if let Some(_) = msg.strip_prefix("serve stop") {
            self.upgrader.stop_serving().await;
        } else if let Some(msg) = msg.strip_prefix("serve ") {
            self.upgrader.serve(msg.into()).await;
        } else if let Some(msg) = msg.strip_prefix("show ") {
            match msg {
                "alias" => {
                    println!("[Management] show alias: {:?}", self.alias);
                }
                "aliases" => {
                    println!("[Management] show aliases: {:?}", self.aliases);
                }
                "discovered" => {
                    println!("[Management] show discovered: {:?}", self.discovered_peers);
                }
                "connected" => {
                    println!("[Management] show connected: {:?}", self.connected_peers);
                }
                "rejected" => {
                    println!("[Management] show rejected: {:?}", self.rejected_peers);
                }
                msg => {
                    println!("[Management] Unknown show command: {}", msg);
                }
            }
        } else if let Some(msg) = msg.strip_prefix("table ") {
            println!("[Management] Entering table mode. Exit with double newline.");
            self.table_mode = true;
            self.table = String::new();
            self.table_receiver = msg.into();
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
                (self.display_show)(msg.payload);
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
fn testing_display_show(mut data: String) {
    println!("[DISPLAY] Sending data to display: {:?}", data);
    unsafe {
        data = data.replace(|c: char| !c.is_ascii(), "");
        toDisplay(data.as_mut_ptr().cast());
    }
}

#[cfg(not(feature = "display"))]
fn testing_display_show(data: String) {
    println!("[DISPLAY] MOCK sending data to display: {:?}", data);
}

#[async_std::main]
async fn main() {
    let mgmt = Management::<NetworkComponent>::new(testing_display_show);

    mgmt.run().await;
}
