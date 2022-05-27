use crate::message::control_message::MessageType;
use crate::message::ControlMessage;
use futures::channel::mpsc;
use futures::channel::mpsc::Receiver;
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use futures::StreamExt;
use p2p_network::NetworkComponent;
use p2p_network::NetworkLayer;
use prost::bytes::Bytes;
use prost::Message;
use libc::char;

//#[link(name = "toDisplay")]
extern {
    fn toDisplay(message: char[]);
}
pub mod message {
    include!(concat!(env!("OUT_DIR"), "/management.control_message.rs"));
}

pub struct Management {
    display_show: fn(data: String),

    send_msg_tx: Sender<Vec<u8>>,
    recv_msg_rx: Receiver<Vec<u8>>,
}

impl Management {
    pub fn new<T: NetworkLayer>(display_show: fn(data: String)) -> Self {
        let (send_msg_tx, send_msg_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(0);
        let (recv_msg_tx, recv_msg_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel(0);

        let m = Management {
            display_show,
            send_msg_tx,
            recv_msg_rx,
        };

        // give channel to network;
        T::run(send_msg_rx, recv_msg_tx);

        return m;
    }

    pub async fn run(mut self) {
        loop {
            let next = self.recv_msg_rx.select_next_some().await;
            self.network_receive(&next).await;
        }
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
        self.send_msg_tx.send(encoded.to_vec()).await.unwrap();
    }

    async fn _handle_message(&mut self, msg: ControlMessage) {
        println!("[Management] Got message: {:?}", msg);

        if msg.test == "ping" {
            self.send(ControlMessage {
                test: "pong".to_string(),
                ..Default::default()
            })
            .await;
        }

        match MessageType::from_i32(msg.message_type) {
            Some(MessageType::DisplayMessage) => {
                (self.display_show)(msg.text.unwrap());
            }
            None => {
                println!("Could not parse message");
            }
        }
    }
}

fn testing_display_show(data: String) {
    println!("[testing] display_show: {:?}", data);
}

#[async_std::main]
async fn main() {
    let mut mgmt = Management::new::<NetworkComponent>(testing_display_show);
    let msg = ControlMessage {
        test: "ping".into(),
        message_type: MessageType::DisplayMessage as i32,
        text: Some("Hello world!".to_string()),
    };

    mgmt._handle_message(msg).await;
    mgmt.run().await;
    // mgmt.receive("CgRwb25n".into());
}
