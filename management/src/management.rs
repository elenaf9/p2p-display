use std::thread;
use std::time;

use crate::message::control_message::MessageType;
use crate::message::ControlMessage;
// #[cfg(feature = "display")]
// use crate::display::toDisplay;
use async_std::io;
use futures::channel::mpsc;
use futures::select;
use futures::AsyncBufReadExt;
use futures::StreamExt;
use p2p_network::NetworkComponent;
use p2p_network::NetworkLayer;
use prost::bytes::Bytes;
use prost::Message;

//#[link(name = "toDisplay")]
#[cfg(feature = "display")]
#[link(name = "display")]
extern "C" {
    pub fn toDisplay(message: *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
pub mod message {
    include!(concat!(env!("OUT_DIR"), "/management.control_message.rs"));
}

pub struct Management<T> {
    display_show: fn(data: String),
    recv_msg_rx: mpsc::Receiver<(String, Vec<u8>)>,

    autorized_senders: Vec<String>,

    network: T,
}

impl<T: NetworkLayer> Management<T> {
    pub fn new(display_show: fn(data: String)) -> Self {
        let (recv_msg_tx, recv_msg_rx) = mpsc::channel(0);

        let network = T::init(recv_msg_tx);

        Management {
            display_show,
            recv_msg_rx,
            network,
            autorized_senders: vec![],
        }
    }

    pub async fn run(mut self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
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
            }
        }
    }

    pub async fn handle_user_input(&mut self, msg: String) {
        if msg.starts_with("send ") {
            self.send(ControlMessage {
                message_type: MessageType::DisplayMessage as i32,
                payload: msg[5..].into(),
                receiver: "".into(),
                sender: self.network.local_peer_id(),
            })
            .await;
        } else if msg.starts_with("whitelist ") {
            let new_peer: String = msg[10..].into();
            let ctrl = ControlMessage {
                message_type: MessageType::AddWhitelistPeer as i32,
                payload: new_peer.clone(),
                receiver: "".into(),
                sender: self.network.local_peer_id(),
            };
            self._handle_message(ctrl.clone()).await;

            thread::sleep(time::Duration::from_millis(200));
            self.send(ctrl).await;
            thread::sleep(time::Duration::from_millis(200));

            let list = self.network.get_whitelisted().await;
            for peer in list {
                self.send(ControlMessage {
                    message_type: MessageType::AddWhitelistPeer as i32,
                    payload: peer,
                    receiver: new_peer.clone(),
                    sender: self.network.local_peer_id(),
                })
                .await;
            }
        } else if msg.starts_with("authorize ") {
            let ctrl = ControlMessage {
                message_type: MessageType::AddWhitelistSender as i32,
                payload: msg[10..].into(),
                receiver: "".into(),
                sender: self.network.local_peer_id(),
            };
            self._handle_message(ctrl.clone()).await;
            self.send(ctrl).await;
        }
    }

    /**
     * Handle an incoming message as as base64-encoded string (for testing).
     */
    pub async fn receive(&mut self, sender: String, msg: String) {
        let bytes = base64::decode(msg).unwrap();
        self.network_receive(sender, &bytes).await;
    }

    /**
     * Receive data from the network.
     */
    pub async fn network_receive(&mut self, _sender: String, data: &[u8]) {
        let bytes = std::boxed::Box::from(data);
        let decoded = ControlMessage::decode(Bytes::from(bytes)).unwrap();
        self._handle_message(decoded).await;
    }

    /**
     * Send a ControlMessage as a base64 encoded string to the network layer.
     *
     * The sender id will automatically be set.
     */
    pub async fn send(&mut self, msg: ControlMessage) {
        let message = ControlMessage {
            sender: self.network.local_peer_id(),
            ..msg
        };
        let encoded = message.encode_to_vec();
        println!(
            "[Management] Sending message of type {:?} to {:?}",
            MessageType::from_i32(message.message_type).unwrap(),
            message
                .receiver
                .get(44..)
                .or_else(|| Some("broadcast"))
                .unwrap(),
        );

        self.network.send_message(encoded.to_vec()).await;
    }

    async fn _handle_message(&mut self, msg: ControlMessage) {
        println!(
            "[Management] Got message of type {:?} from {:?}",
            MessageType::from_i32(msg.message_type).unwrap(),
            &msg.sender.get(44..).or_else(|| Some("broadcast")).unwrap(),
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
