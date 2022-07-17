use std::collections::HashMap;

pub struct Dht {
    /// Online whitelisted peers.
    online_peers: Vec<String>,

    /// Peer specific content.
    peer_content: HashMap<String, String>,

    /// Broadcasted content.
    broadcast_content: Option<String>,
}

impl Dht {
    pub fn new(own_id: String) -> Self {
        Dht {
            online_peers: vec![own_id],
            peer_content: HashMap::new(),
            broadcast_content: None,
        }
    }

    pub fn add_peer(&mut self, peer: String) {
        if !self.online_peers.contains(&peer) {
            self.online_peers.push(peer);
            self.online_peers.sort()
        }
    }

    pub fn remove_peer(&mut self, peer: &String) {
        self.online_peers.retain(|p| p != peer);
    }

    pub fn store(&mut self, target: String, data: String) {
        self.peer_content.insert(target, data);
    }

    pub fn store_broadcast_content(&mut self, data: String) {
        for value in self.peer_content.values_mut() {
            *value = data.clone()
        }
        self.broadcast_content = Some(data);
    }

    pub fn get_closest_peers(&mut self, target: &String) -> Vec<String> {
        let index = self
            .online_peers
            .iter()
            .position(|p| p >= target)
            .unwrap_or(0);
        let closest = self
            .online_peers
            .get(index)
            .expect("List is not empty.")
            .clone();
        let mut list = vec![closest];
        if let Some(next_closest) = self.online_peers.get((index + 1) % self.online_peers.len()) {
            if next_closest != &list[0] {
                list.push(next_closest.clone())
            }
        }
        list
    }

    pub fn get_closest_other(&mut self, target: &String) -> Vec<String> {
        let iter = self.online_peers.iter().filter(|p| p != &target);
        let index = iter.clone().position(|p| p > target).unwrap_or(0);
        let mut list = Vec::new();
        match self.online_peers.get(index) {
            Some(peer) => list.push(peer.clone()),
            None => return Vec::new(),
        }
        if let Some(next_closest) = self.online_peers.get((index + 1) % self.online_peers.len()) {
            if next_closest != &list[0] {
                list.push(next_closest.clone())
            }
        }
        list
    }

    pub fn get_content(&self, target: &String) -> Option<String> {
        self.peer_content
            .get(target)
            .cloned()
            .or_else(|| self.broadcast_content.clone())
    }

    pub fn get_peers(&self) -> Vec<String> {
        self.online_peers.clone()
    }
}
