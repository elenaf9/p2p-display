#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures::{channel::mpsc, SinkExt};
use management::{Management, UserCommand};
use tauri::State;

struct InputHandler {
    user_input_tx: mpsc::Sender<UserCommand>,
}

impl InputHandler {
    fn new() -> Self {
        let (user_input_tx, user_input_rx) = mpsc::channel(0);
        tauri::async_runtime::spawn(Management::new(user_input_rx).run());
        InputHandler { user_input_tx }
    }
}

#[tauri::command]
fn handle_input(input: String, handler: State<InputHandler>) {
    let mut user_input_tx = handler.user_input_tx.clone();
    let command = UserCommand::SendMsg {
        peer: String::new(),
        message: input
    };
    tauri::async_runtime::block_on(user_input_tx.send(command)).unwrap();
}

fn main() {
    tauri::Builder::default()
        .manage(InputHandler::new())
        .invoke_handler(tauri::generate_handler![handle_input])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
