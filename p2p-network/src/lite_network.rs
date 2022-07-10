use std::{net::{TcpListener, IpAddr, TcpStream}, collections::HashMap};
use futures::{executor::LocalPool, channel::mpsc, future::{poll_fn, FutureExt}, task::LocalSpawn, select};
use async_io::Async;

use crate::network::Command;

fn lite_network(ip: IpAddr, port: u16, command_rx: mpsc::Receiver<Command>, message_tx: mpsc::Sender<(String, Vec<u8>)>) {
    let mut pool = LocalPool::new();
    let mut listener = Async::new(TcpListener::bind((ip, port)).unwrap()).unwrap();
    let mut dialer = Async::new(TcpListener::bind("0.0.0.0:0").unwrap()).unwrap();
    let mut streams = Vec::new();
    let mut addresses: HashMap<String, (IpAddr, u16)> = HashMap::new();
    let mut whitelisted: Vec<String> = Vec::new();

    loop {
        // TODO: poll and handles commands, drive streams
        let new_stream = pool.run_until(poll_listener(&mut listener));
        streams.push(new_stream);
    }
}


async fn poll_listener(listener: &mut Async<TcpListener>) -> Async<TcpStream>{
    let (stream, _) = loop {
        match poll_fn(|cx| listener.poll_readable(cx)).await {
            Err(_) => {},
            Ok(()) => match listener.accept().now_or_never() {
                Some(Ok(res)) => break res,
                _ => {}
            },
        }
    };
    stream
}
