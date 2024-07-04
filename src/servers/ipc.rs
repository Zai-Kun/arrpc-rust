use crate::consts::MOCK_USER;
use crate::handle_activity;
use scopeguard::defer;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, i32};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

fn get_socket_path_unix() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .or_else(|_| std::env::var("TMPDIR"))
        .or_else(|_| std::env::var("TMP"))
        .or_else(|_| std::env::var("TEMP"))
        .unwrap_or_else(|_| String::from("/tmp"));
    let path = PathBuf::from(runtime_dir).join("discord-ipc-0");
    if path.exists() {
        fs::remove_file(&path).expect("Failed to remove socket file");
    }
    path
}

fn get_socket_path() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(r"\\?\pipe\discord-ipc")
    }
    #[cfg(unix)]
    get_socket_path_unix()
}

async fn handle_connection_unix(mut stream: UnixStream, tx: UnboundedSender<Message>) {
    stream
        .write_all(&MOCK_USER.trim().len().to_be_bytes())
        .await
        .expect("Error writing to stream");
    stream
        .write_all(MOCK_USER.trim().as_bytes())
        .await
        .expect("Error writing to stream");
    println!("IPC: conected");
    let mut client_id: Option<String> = None;
    let last_pid: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
    defer!({
        if let Some(some) = *last_pid.clone().lock().unwrap() {
            handle_activity::clear_activity(some, tx.clone())
        }
    });
    loop {
        let mut header = [0; 8];

        stream.read_exact(&mut header).await.unwrap();

        let data_size_bytes = &header[4..8];
        let data_size = i32::from_le_bytes(data_size_bytes.try_into().unwrap());
        let mut buffer = (0..data_size).map(|_| 0).collect::<Vec<u8>>();
        stream.read_exact(&mut buffer).await.unwrap();

        let data = String::from_utf8_lossy(&buffer).trim().to_string();
        let decoded_data: Value = if let Ok(ok) = serde_json::from_str(&data) {
            ok
        } else {
            continue;
        };
        if let Some(some) = decoded_data.get("client_id") {
            client_id = Some(some.as_str().unwrap().to_string());
            continue;
        }

        if let Some(some) = decoded_data.get("cmd") {
            if let Some(some) = some.as_str() {
                if some == "SET_ACTIVITY" {
                    *last_pid.lock().unwrap() = handle_activity::handle(
                        decoded_data.clone(),
                        if let Some(some) = client_id.as_ref() {
                            some
                        } else {
                            "0"
                        },
                        tx.clone(),
                    );

                    let mut resp: HashMap<String, Value> =
                        serde_json::from_value(decoded_data).unwrap();
                    resp.insert("evt".to_string(), Value::Null);
                    let resp = serde_json::to_string(&resp).unwrap();
                    stream.write_all(&resp.len().to_be_bytes()).await.unwrap();
                    stream.write_all(&resp.as_bytes()).await.unwrap();
                }
            }
        }
    }
}

pub async fn start_server(tx: UnboundedSender<Message>) {
    let socket_path = get_socket_path();
    #[cfg(windows)]
    {
        println!("IPC not implimented for windows (yet (probably))");
        return;
    }
    #[cfg(unix)]
    let listener =
        tokio::net::UnixListener::bind(&socket_path).expect("Failed to bind to Unix socket");

    println!("IPC started at {}", socket_path.display());

    loop {
        let (socket, _) = listener
            .accept()
            .await
            .expect("Failed to accept connection");
        tokio::task::spawn(handle_connection_unix(socket, tx.clone()));
    }
}
