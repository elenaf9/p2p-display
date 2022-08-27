pub trait NetworkLayer {
  // Create a new network.
  // Inbound messages from remote peers are forwarded as 
  // (sender, message) tuple through `in_message_tx`.
  /// Optionally the identity private key may be loaded 
  /// from a file. It is expected that the key is an 
  /// OpenSSL ed25519 private key in PEM format.
  fn init(
    private_key: Option<&Path>,
    in_message_tx: mpsc::Sender<(String, Vec<u8>, bool)>,
    event_tx: mpsc::Sender<NetworkEvent>,
  ) -> Self;
  /// Our own unique id in the network.
  fn local_peer_id(&self) -> String;
  /// Publish a message to the whole network.
  async fn publish_message(&mut self, message: Vec<u8>);
  /// Send a direct message to one peer.
  async fn send_message(
    &mut self,
    peer: String, 
    message: Vec<u8>
  );
  /// Get the list of currently whitelisted peers.
  /// This is the list of peers for which we allow sending
  /// and receiving messages on the network layer.
  async fn get_whitelisted(&mut self) -> Vec<String>;
  /// Add a peer to our local whitelist.
  async fn add_whitelisted(&mut self, peer: String);
  /// Remove a peer from our local whitelist.
  async fn remove_whitelisted(&mut self, peer: String);
}