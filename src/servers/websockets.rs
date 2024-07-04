use crate::consts::MOCK_USER;
use crate::handle_activity;
use futures_util::{SinkExt, StreamExt};
use scopeguard::defer;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

async fn handle_connection(
    raw_stream: tokio::net::TcpStream,
    _addr: SocketAddr,
    sender: mpsc::UnboundedSender<Message>,
) {
    println!("websockets: {} connected", raw_stream.peer_addr().unwrap());
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake");

    let (mut write, mut read) = ws_stream.split();
    let _ = write.send(Message::text(MOCK_USER)).await;

    let mut client_id: Option<String> = None;
    let last_pid: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
    defer!({
        if let Some(some) = *last_pid.clone().lock().unwrap() {
            handle_activity::clear_activity(some, sender.clone())
        }
    });
    while let Some(msg) = read.next().await {
        let msg = msg.expect("Error reading message");
        if !msg.is_text() {
            continue;
        }
        let msg = msg.to_string();
        let decoded_data: Value = if let Ok(ok) = serde_json::from_str(&msg) {
            ok
        } else {
            continue;
        };
        if let Some(some) = decoded_data.get("client_id") {
            client_id = Some(some.to_string());
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
                        sender.clone(),
                    );
                    let mut resp: HashMap<String, Value> =
                        serde_json::from_value(decoded_data).unwrap();
                    resp.insert("evt".to_string(), Value::Null);
                    let _ = write
                        .send(Message::text(serde_json::to_string(&resp).unwrap()))
                        .await;
                }
            }
        }
    }
}

pub async fn start_server(sender: mpsc::UnboundedSender<Message>) {
    let addr = "127.0.0.1:6463";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Can't bind to address");

    println!("Server started on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, sender.clone()));
    }
}
