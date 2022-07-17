use std::collections::HashMap;

pub struct Dht {
    own_id: String,
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
            own_id: own_id.clone(),
            online_peers: vec![own_id],
            peer_content: HashMap::new(),
            broadcast_content: None,
        }
    }

    // Add peer to the list of online peers.
    // Returns list of content to be republished in a StoreMessage request.
    // Format: (request_target, Vec<(data_owner, data)>)
    pub fn add_peer(&mut self, peer: String) -> Option<(String, Vec<(Option<String>, String)>)> {
        let peer_index = match self.online_peers.binary_search(&peer) {
            Ok(_) => return None,
            Err(index) => index,
        };
        self.online_peers.insert(peer_index, peer.clone());
        let own_index = self
            .online_peers
            .binary_search(&self.own_id)
            .expect("Own Id is always included");
        let online_count = self.online_peers.len();
        if online_count <= 2 {
            let mut content: Vec<_> = self
                .peer_content
                .clone()
                .into_iter()
                .map(|(r, d)| (Some(r), d))
                .collect();
            if let Some(broadcast) = self.broadcast_content.clone() {
                content.push((None, broadcast));
            }
            return Some((peer, content));
        }
        let prev_index = (own_index + online_count - 1) % online_count;
        let prev_prev_index = (own_index + online_count - 2) % online_count;
        let prev_prev_prev_index = (own_index + online_count - 3) % online_count;
        let republish = if peer_index == prev_index {
            // Peer will be inserted into the list directly in front of us.
            // Part of the data for which we were backup can now be stored at the
            // new peer.
            //
            // E.g. We are 'C' in sorted list A-B-C and 'B2' connects so that the new
            // order is A-B-B2-C:
            // Until now we stored the data between A-B and B-C. Now the data between A-B
            // is stored at B2. We now only need to stored the one between B-B2 and B2-C,
            // which we already do.
            let a_id = self.online_peers.get(prev_prev_prev_index).unwrap().clone();
            let b_id = self.online_peers.get(prev_prev_index).unwrap().clone();
            let peer_content = self.get_content_between(&a_id, &b_id);
            for (k, _) in &peer_content {
                self.peer_content.remove(k);
            }
            let mut content: Vec<_> = peer_content
                .clone()
                .into_iter()
                .map(|(r, d)| (Some(r), d))
                .collect();
            if let Some(broadcast) = self.broadcast_content.clone() {
                content.push((None, broadcast));
            }
            Some((peer.clone(), content))
        } else if peer_index == prev_prev_index {
            // Peer will be inserted into the list two indexes in front of us.
            // Part of the data for which we were backup can now be stored at the
            // new peer.
            //
            // E.g. We are 'C' in sorted list A-B-C and 'A2' connects so that the new
            // order is A-A2-B-C:
            // Until we stored the data between A-B and B-C. Now the data between A-A2
            // is stored at B2. We now only need to stored the one between A2-B and B-C,
            // which we already do.
            let a_id = self.online_peers.get(prev_prev_prev_index).unwrap().clone();
            let peer_content = self.get_content_between(&a_id, &peer);
            for (k, _) in &peer_content {
                self.peer_content.remove(k);
            }
            let mut content: Vec<_> = peer_content
                .clone()
                .into_iter()
                .map(|(r, d)| (Some(r), d))
                .collect();
            if let Some(broadcast) = self.broadcast_content.clone() {
                content.push((None, broadcast));
            }
            Some((peer.clone(), content))
        } else {
            None
        };
        republish
    }

    // Remove peer from the list of online peers.
    // Returns list of content to be republished in a StoreMessage request.
    // Format: (request_target, Vec<(data_owner, data)>)
    pub fn remove_peer(&mut self, peer: &String) -> Option<(String, Vec<(String, String)>)> {
        let peer_index = match self.online_peers.binary_search(peer) {
            Ok(i) => i,
            Err(_) => return None,
        };
        let own_index = self
            .online_peers
            .binary_search(&self.own_id)
            .expect("Own Id is always included");
        self.online_peers.retain(|p| p != peer);
        let online_count = self.online_peers.len();
        let prev_index = (own_index + online_count - 1) % online_count;
        let next_index = (own_index + online_count + 1) % online_count;
        if peer_index == prev_index {
            // Peer preceeding us in the list disconnected.
            // Republish all data for which the disconnected peer was "first" backup (and thus we "second").
            //
            // E.g. We are 'C' in sorted list A-B-C-D-E and 'B' disconnects:
            // Until now peer D only stores the data for between B-C and C-D. New order is A-C-D-E,
            // therefore D needs to also store the data that used to be between A and B.
            let a_id = self.online_peers.get(prev_index).unwrap().clone();
            let content = self.get_content_between(&a_id, peer);
            let d_id = self.online_peers.get(next_index).unwrap();
            Some((d_id.clone(), content))
        } else if peer_index == next_index {
            // Peer succeeding us in the list disconnected.
            // Republish all data for which the disconnected peer was "second" backup (and thus we "first").
            //
            // E.g. We are 'C' in sorted list A-B-C-D-E and 'D' disconnects:
            // Until now peer E only stores the data for between C-D and D-E. New order is A-B-C-E,
            // therefore E needs to also store the data of peers between B and C.
            let b_id = self.online_peers.get(prev_index).unwrap().clone();
            let content = self.get_content_between(&b_id, &self.own_id.clone());
            let e_id = self.online_peers.get(next_index).unwrap();
            Some((e_id.clone(), content))
        } else {
            None
        }
    }

    // Get the content stored for peers whose id is <= start and < end.
    pub fn get_content_between(&mut self, start: &String, end: &String) -> Vec<(String, String)> {
        let mut keys = self.peer_content.keys().collect::<Vec<_>>();
        keys.sort();
        let start = keys.partition_point(|p| p < &start);
        keys.rotate_left(start);
        keys.into_iter()
            .take_while(|p| p <= &end)
            .map(|k| (k.clone(), self.peer_content.get(k).unwrap().clone()))
            .collect()
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
            .clone()
            .position(|p| p > target)
            .unwrap_or(0);
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

    pub fn get_online_peers(&self) -> &Vec<String> {
        &self.online_peers
    }
}
