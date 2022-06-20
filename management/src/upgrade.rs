use std::{
    fs::{self},
    io::{Read, Write},
    net::TcpStream,
    os::unix::prelude::{OpenOptionsExt, PermissionsExt},
    process::exit,
};

use async_std::{
    fs::File,
    io::{ReadExt, WriteExt},
    net::TcpListener,
    prelude::StreamExt,
    task::JoinHandle,
};

pub struct UpgradeServer {
    handle: Option<JoinHandle<()>>,
}

const UPGRADE_SERVER_PORT: &str = "9803";

impl UpgradeServer {
    pub fn new() -> Self {
        UpgradeServer { handle: None }
    }

    pub fn upgrade_binary(network_addr: String) -> Result<(), ()> {
        let current_path = std::env::current_exe().unwrap();
        let update_path = current_path.with_file_name(".management.update");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o777)
            .open(&update_path)
            .unwrap();
        file.metadata().unwrap().permissions().set_mode(0o777);

        let mut tcp = TcpStream::connect(network_addr).unwrap();

        let mut buf = [0; 4096];
        loop {
            let n = tcp.read(&mut buf).unwrap();

            if n == 0 {
                break;
            }

            let _ = file.write(&buf[..n]);
        }

        let _ = fs::remove_file(&current_path).unwrap();
        let _ = fs::rename(&update_path, &current_path);

        println!("[UpgradeServer] Exiting to apply update...");

        exit(0);
    }

    pub async fn serve(&mut self, file_path: String) {
        let file = File::open(&file_path).await;

        if file.is_err() {
            println!("[UpgradeServer] Could not serve file {:?}", &file_path);
            return;
        }

        let network_addr: String = String::from("0.0.0.0:") + UPGRADE_SERVER_PORT;
        let listener = TcpListener::bind(&network_addr).await;
        if listener.is_err() {
            println!("[UpgradeServer] Could not listen on {}", &network_addr);
            return;
        }
        println!("[UpgradeServer] Listening on {:?}...", &network_addr);

        let handle = async_std::task::spawn(async {
            let file_path = file_path;
            let listener = listener.unwrap();

            let mut incoming = listener.incoming();

            while let Some(Ok(mut stream)) = incoming.next().await {
                println!("[UpgradeServer] Serving file...");

                let mut file = File::open(&file_path).await.unwrap();
                let mut buf = [0; 4096];

                loop {
                    let n = file.read(&mut buf).await.unwrap();

                    if n == 0 {
                        break;
                    }

                    let _ = stream.write_all(&buf[..n]).await;
                }

                let _ = stream.flush().await;

                println!("[UpgradeServer] File served.");
            }
        });
        self.handle = Some(handle);
    }

    pub async fn stop_serving(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.cancel().await;
            println!("[UpgradeServer] Stopped serving.");
        }
    }
}
