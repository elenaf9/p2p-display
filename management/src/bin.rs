use async_std::io;
use futures::{channel::mpsc, prelude::*};
use management::Management;

fn main() {
    let (mut user_input_tx, user_input_rx) = mpsc::channel(0);
    async_std::task::spawn(async move {
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            let message = stdin.select_next_some().await.unwrap();
            user_input_tx.send(message).await.unwrap();
        }
    });
    async_std::task::block_on(Management::new(user_input_rx).run())
}
