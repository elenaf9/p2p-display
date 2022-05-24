mod network;

use futures::channel::mpsc;
use network::Network;

pub struct NetworkComponent;

pub trait NetworkLayer {
    fn run(out_message_rx: mpsc::Receiver<Vec<u8>>, in_message_tx: mpsc::Sender<Vec<u8>>);
}

impl NetworkLayer for NetworkComponent {
    fn run(out_message_rx: mpsc::Receiver<Vec<u8>>, in_message_tx: mpsc::Sender<Vec<u8>>) {
        async_std::task::spawn(async {
            // All logic is implement in our `network` mod.
            // Refer to its docs for more info on the below method calls.
            let mut network = Network::new(out_message_rx, in_message_tx).await;
            network.start_listening();
            network.subscribe();
            network.run().await
        });
    }
}
