#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures::{
    channel::{mpsc, oneshot},
    SinkExt,
};
use management::{Management, UserCommand};
use tauri::{State, Submenu, CustomMenuItem, Menu, MenuItem, WindowBuilder};

struct CommandHandler {
    user_cmd_tx: mpsc::Sender<UserCommand>,
}

impl CommandHandler {
    fn new() -> Self {
        let (user_cmd_tx, user_cmd_rx) = mpsc::channel(0);
        tauri::async_runtime::spawn(Management::new(user_cmd_rx).run());
        CommandHandler { user_cmd_tx }
    }
}

#[tauri::command]
fn publish_message(message: String, peer: Option<String>, handler: State<CommandHandler>) {
    let mut user_cmd_tx = handler.user_cmd_tx.clone();
    let command = UserCommand::SendMsg { peer, message };
    tauri::async_runtime::block_on(user_cmd_tx.send(command)).unwrap();
}

#[tauri::command]
fn whitelist(peer: String, handler: State<CommandHandler>) {
    let mut user_cmd_tx = handler.user_cmd_tx.clone();
    let command = UserCommand::Whitelist(peer);
    tauri::async_runtime::block_on(user_cmd_tx.send(command)).unwrap();
}

#[tauri::command]
fn authorize(peer: String, handler: State<CommandHandler>) {
    let mut user_cmd_tx = handler.user_cmd_tx.clone();
    let command = UserCommand::Authorize(peer);
    tauri::async_runtime::block_on(user_cmd_tx.send(command)).unwrap();
}

#[tauri::command]
fn alias(alias: String, handler: State<CommandHandler>) {
    let mut user_cmd_tx = handler.user_cmd_tx.clone();
    let command = UserCommand::Alias(alias);
    tauri::async_runtime::block_on(user_cmd_tx.send(command)).unwrap();
}

#[tauri::command]
fn get_local_id(handler: State<CommandHandler>) -> String {
    let mut user_cmd_tx = handler.user_cmd_tx.clone();
    let (tx, rx) = oneshot::channel();
    let command = UserCommand::GetPeerId(tx);
    let fut = async {
        user_cmd_tx.send(command).await.unwrap();
        rx.await.unwrap()
    };
    tauri::async_runtime::block_on(fut)
}

fn main() {
    tauri::Builder::default()
        .menu(tauri::Menu::os_default("Digital Fax"))
        .manage(CommandHandler::new())
        .invoke_handler(tauri::generate_handler![
            publish_message,
            whitelist,
            authorize,
            get_local_id,
            alias
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
