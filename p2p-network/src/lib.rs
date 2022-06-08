mod network;

use std::str::FromStr;

use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use libp2p::{identity, PeerId};
use network::{Command, Network};

#[derive(Clone)]
pub struct NetworkComponent {
    command_tx: mpsc::Sender<Command>,
    local_peer_id: PeerId,
}

#[async_trait]
pub trait NetworkLayer {
    fn init(in_message_tx: mpsc::Sender<(String, Vec<u8>)>) -> Self;

    fn local_peer_id(&self) -> String;

    async fn send_message(&mut self, message: Vec<u8>);
    async fn get_whitelisted(&mut self) -> Vec<String>;
    async fn add_whitelisted(&mut self, peer: String);
    async fn remove_whitelisted(&mut self, peer: String);
}

#[async_trait]
impl NetworkLayer for NetworkComponent {
    fn init(in_message_tx: mpsc::Sender<(String, Vec<u8>)>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(0);

        // Authentication keypair.
        // Used to derive a unique PeerId and the keypair for encryption on the
        // Transport layer with the Noise protocol (https://noiseprotocol.org/noise.html).
        let keypair = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from_public_key(&keypair.public());

        async_std::task::spawn(async {
            // All logic is implement in our `network` mod.
            // Refer to its docs for more info on the below method calls.
            let mut network = Network::new(keypair, command_rx, in_message_tx).await;
            network.start_listening();
            network.subscribe();
            network.run().await
        });
        NetworkComponent {
            command_tx,
            local_peer_id,
        }
    }

    fn local_peer_id(&self) -> String {
        self.local_peer_id.to_base58()
    }

    async fn send_message(&mut self, message: Vec<u8>) {
        let command = Command::SendMessage { message };
        self.command_tx.send(command).await.unwrap();
    }

    async fn get_whitelisted(&mut self) -> Vec<String> {
        let (tx, rx) = oneshot::channel();
        let command = Command::GetWhitelisted { tx };
        self.command_tx.send(command).await.unwrap();
        rx.await
            .unwrap()
            .into_iter()
            .map(|id| id.to_base58())
            .collect()
    }

    async fn add_whitelisted(&mut self, peer: String) {
        let command = Command::AddWhitelisted {
            peer: PeerId::from_str(&peer).unwrap(),
        };
        self.command_tx.send(command).await.unwrap();
    }
    async fn remove_whitelisted(&mut self, peer: String) {
        let command = Command::RemoveWhitelisted {
            peer: PeerId::from_str(&peer).unwrap(),
        };
        self.command_tx.send(command).await.unwrap();
    }
}
