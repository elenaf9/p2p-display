use async_std::io;
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use management::{Management, UserCommand};

struct Table {
    peer: Option<String>,
    message: String,
}

async fn handle_user_input(
    cmd_tx: &mut mpsc::Sender<UserCommand>,
    table: &mut Option<Table>,
    msg: String,
) {
    if let Some(t) = table.as_mut() {
        if msg.is_empty() {
            let table = table.take().unwrap();
            cmd_tx
                .send(UserCommand::SendMsg {
                    peer: table.peer,
                    message: table.message.into(),
                })
                .await
                .unwrap();
        } else {
            t.message.push_str(&msg);
            t.message.push_str("\n");
        }
        return;
    }
    if let Some(msg) = msg.strip_prefix("table ") {
        println!("[Management] Entering table mode. Exit with double newline.");
        let _ = table.insert(Table {
            peer: (!msg.is_empty()).then(|| msg.into()),
            message: String::new(),
        });
        return;
    }
    let msg_clone = msg.clone();
    let mut res_fut = None;
    let command = if let Some(msg) = msg.strip_prefix("send ") {
        UserCommand::SendMsg {
            peer: None,
            message: msg.into(),
        }
    } else if let Some(msg) = msg.strip_prefix("sendto ") {
        let parts = msg.split_once(" ").unwrap();
        UserCommand::SendMsg {
            peer: Some(parts.0.into()),
            message: parts.1.into(),
        }
    } else if let Some(msg) = msg.strip_prefix("whitelist ") {
        UserCommand::Whitelist(msg.into())
    } else if let Some(msg) = msg.strip_prefix("authorize ") {
        UserCommand::Authorize(msg.into())
    } else if let Some(msg) = msg.strip_prefix("alias ") {
        UserCommand::Alias(msg.into())
    } else if let Some(msg) = msg.strip_prefix("upgrade self ") {
        UserCommand::UpgradeSelf(msg.into())
    } else if let Some(msg) = msg.strip_prefix("upgrade ") {
        let parts = msg.split_once(" ").unwrap();
        UserCommand::Upgrade(parts.0.into(), parts.1.into())
    } else if let Some(_) = msg.strip_prefix("serve stop") {
        UserCommand::ServeStop
    } else if let Some(msg) = msg.strip_prefix("serve ") {
        UserCommand::Serve(msg.into())
    } else if let Some(msg) = msg.strip_prefix("show ") {
        match msg {
            "alias" => {
                let (tx, rx) = oneshot::channel();
                let _ = res_fut.insert(rx.map_ok(|res| format!("{:?}", res)).boxed());
                UserCommand::GetAlias(tx)
            }
            "aliases" => {
                let (tx, rx) = oneshot::channel();
                let _ = res_fut.insert(rx.map_ok(|res| format!("{:?}", res)).boxed());
                UserCommand::GetAliases(tx)
            }
            "discovered" => {
                let (tx, rx) = oneshot::channel();
                let _ = res_fut.insert(rx.map_ok(|res| format!("{:?}", res)).boxed());
                UserCommand::GetDiscovered(tx)
            }
            "connected" => {
                let (tx, rx) = oneshot::channel();
                let _ = res_fut.insert(rx.map_ok(|res| format!("{:?}", res)).boxed());
                UserCommand::GetConnected(tx)
            }
            "rejected" => {
                let (tx, rx) = oneshot::channel();
                let _ = res_fut.insert(rx.map_ok(|res| format!("{:?}", res)).boxed());
                UserCommand::GetRejected(tx)
            }
            _ => {
                println!("[Management] Unknown show command: {}", msg);
                return;
            }
        }
    } else {
        return;
    };
    cmd_tx.send(command).await.unwrap();
    let res = match res_fut.take() {
        Some(res_fut) => res_fut.await.unwrap(),
        None => return,
    };
    println!("[Management] {:?}: {:?}", msg_clone, res);
}

fn main() {
    let (mut user_input_tx, user_input_rx) = mpsc::channel(0);
    async_std::task::spawn(async move {
        let mut table = None;
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            let message = stdin.select_next_some().await.unwrap();
            handle_user_input(&mut user_input_tx, &mut table, message).await;
        }
    });
    async_std::task::block_on(Management::new(user_input_rx).run())
}
