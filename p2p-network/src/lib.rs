mod network;

use std::str::FromStr;

use async_trait::async_trait;
use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use libp2p::PeerId;
use network::{Command, Network};

#[derive(Clone)]
pub struct NetworkComponent {
    command_tx: mpsc::Sender<Command>,
}

#[async_trait]
pub trait NetworkLayer {
    fn init(in_message_tx: mpsc::Sender<Vec<u8>>) -> Self;

    async fn send_message(&mut self, message: Vec<u8>);
    async fn get_whitelisted(&mut self) -> Vec<String>;
    async fn update_whitelisted(&mut self, peers: Vec<String>);
}

#[async_trait]
impl NetworkLayer for NetworkComponent {
    fn init(in_message_tx: mpsc::Sender<Vec<u8>>) -> Self {
        let (command_tx, command_rx) = mpsc::channel(0);

        async_std::task::spawn(async {
            // All logic is implement in our `network` mod.
            // Refer to its docs for more info on the below method calls.
            let mut network = Network::new(command_rx, in_message_tx).await;
            network.start_listening();
            network.subscribe();
            network.run().await
        });
        NetworkComponent { command_tx }
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

    async fn update_whitelisted(&mut self, peers: Vec<String>) {
        let peers = peers
            .into_iter()
            .map(|id| PeerId::from_str(&id))
            .collect::<Result<_, _>>()
            .unwrap();
        let command = Command::UpdateWhiteListed { peers };
        self.command_tx.send(command).await.unwrap();
    }
}
