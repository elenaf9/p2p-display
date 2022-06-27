mod network;

use std::{path::Path, str::FromStr};

use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use libp2p::{
    identity::{self, ed25519},
    PeerId,
};
use network::{Command, Network};

pub enum NetworkEvent {
    ConnectionEstablished { peer: String },
    ConnectionRejected { peer: String },
    PeerDiscovered { peer: String },
    PeerExpired { peer: String },
    NewListenAddress { addr: String },
}

pub struct NetworkComponent {
    command_tx: mpsc::Sender<Command>,
    local_peer_id: PeerId,
}

#[async_trait]
pub trait NetworkLayer {
    /// Create a new network.
    ///
    /// Inbound messages from remote peers are forwarded as (sender, message) tuple
    /// through `in_message_tx`.
    ///
    /// Optionally the identity private key may be loaded from a file.
    /// It is expected that the key is an OpenSSL ed25519 private key in PEM format.
    fn init(
        private_key: Option<&Path>,
        in_message_tx: mpsc::Sender<(String, Vec<u8>)>,
        event_tx: mpsc::Sender<NetworkEvent>,
    ) -> Self;

    fn local_peer_id(&self) -> String;

    async fn send_message(&mut self, message: Vec<u8>);
    async fn get_whitelisted(&mut self) -> Vec<String>;
    async fn add_whitelisted(&mut self, peer: String);
    async fn remove_whitelisted(&mut self, peer: String);
}

#[async_trait]
impl NetworkLayer for NetworkComponent {
    fn init(
        private_key: Option<&Path>,
        in_message_tx: mpsc::Sender<(String, Vec<u8>)>,
        event_tx: mpsc::Sender<NetworkEvent>,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::channel(0);

        // Load an ed25519 keypair from file or generate a new one.
        //
        // Used to derive a unique PeerId and the keypair for encryption on the
        // Transport layer with the Noise protocol (https://noiseprotocol.org/noise.html).
        //
        let keypair = private_key
            .and_then(|path| {
                let sk_bytes = std::fs::read(path).ok()?;
                let static_secret =
                    curve25519_parser::parse_openssl_25519_privkey(&sk_bytes).ok()?;
                let identity_ed25199_sk =
                    ed25519::SecretKey::from_bytes(static_secret.to_bytes()).ok()?;
                Some(identity::Keypair::Ed25519(identity_ed25199_sk.into()))
            })
            .unwrap_or_else(identity::Keypair::generate_ed25519);
        let local_peer_id = PeerId::from_public_key(&keypair.public());

        async_std::task::spawn(async {
            // All logic is implement in our `network` mod.
            // Refer to its docs for more info on the below method calls.
            let mut network = Network::new(keypair, command_rx, in_message_tx, event_tx).await;
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
