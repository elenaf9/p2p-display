use crate::message::control_message::MessageType;
use crate::message::ControlMessage;
// #[cfg(feature = "display")]
// use crate::display::toDisplay;
use async_std::io;
use futures::channel::mpsc;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::select;
use futures::AsyncBufReadExt;
use futures::StreamExt;
use p2p_network::NetworkComponent;
use p2p_network::NetworkLayer;
use prost::bytes::Bytes;
use prost::Message;


//#[link(name = "toDisplay")]
#[cfg(feature = "display")]
extern "C" {
    fn toDisplay(message: String);
}
pub mod message {
    include!(concat!(env!("OUT_DIR"), "/management.control_message.rs"));
}

pub struct Management<T> {
    display_show: fn(data: String),
    recv_msg_rx: Receiver<Vec<u8>>,

    network: T,
}

impl<T: NetworkLayer> Management<T> {
    pub fn new(display_show: fn(data: String)) -> Self {
        let (recv_msg_tx, recv_msg_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(0);

        let network = T::init(recv_msg_tx);

        let m = Management {
            display_show,
            recv_msg_rx,
            network,
        };

        return m;
    }

    pub async fn run(mut self) {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            // `Select` is a macro that simultaneously polls items.
            select! {
                // Poll the swarm for events.
                // Even if we would not care about the event, we have to poll the
                // swarm for it to make any progress.
                event = self.recv_msg_rx.select_next_some() => {
                    self.network_receive(&event).await;
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
        let msg = ControlMessage {
            message_type: MessageType::DisplayMessage as i32,
            text: msg.to_string(),
        };
        self.send(msg).await;
    }

    /**
     * Handle an incoming message as as base64-encoded string (for testing).
     */
    pub async fn receive(&mut self, msg: String) {
        let bytes = base64::decode(msg).unwrap();
        self.network_receive(&bytes).await;
    }

    /**
     * Receive data from the network.
     */
    pub async fn network_receive(&mut self, data: &[u8]) {
        let bytes = std::boxed::Box::from(data);
        let decoded = ControlMessage::decode(Bytes::from(bytes)).unwrap();
        self._handle_message(decoded).await;
    }

    /**
     * Send a ControlMessage as a base64 encoded string to the network layer.
     */
    pub async fn send(&mut self, msg: ControlMessage) {
        let encoded = msg.encode_to_vec();
        let base64_encoded = base64::encode(&encoded);
        println!(
            "[Management] Sending message: {:?} (base64: {:?})",
            msg, base64_encoded,
        );

        // (self.network_send)(&encoded);
        self.network.send_message(encoded.to_vec()).await;
    }

    async fn _handle_message(&mut self, msg: ControlMessage) {
        println!("[Management] Got message: {:?}", msg);

        match MessageType::from_i32(msg.message_type) {
            Some(MessageType::DisplayMessage) => {
                (self.display_show)(msg.text);
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
