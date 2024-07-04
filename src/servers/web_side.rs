use core::panic;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

async fn handle_connection(
    raw_stream: tokio::net::TcpStream,
    addr: SocketAddr,
    active_conn: Arc<Mutex<bool>>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,
) {
    {
        let mut active = active_conn.lock().await;
        if *active {
            println!("Connection refused from {}", addr);
            return;
        }
        *active = true;
    }

    let mut receiver = receiver.lock().await;
    let ws_stream = accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake");

    let (mut write, mut read) = ws_stream.split();
    loop {
        tokio::select! {
            msg_from_receiver = receiver.recv() => {
                if let Some(msg) = msg_from_receiver {
                    if msg.is_text() || msg.is_binary() {
                        if let Err(err) = write.send(msg).await {
                            let mut active = active_conn.lock().await;
                            *active = false;
                            panic!("{}", err);
                        }
                    }
            } else {
                break;
            }
        }
            msg_from_client = read.next() => {
                if let Some(msg) = msg_from_client {
                    if let Err(_) = msg {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }
    let mut active = active_conn.lock().await;
    *active = false;
    println!("{} disconnected", addr);
}

pub async fn start_server(receiver: mpsc::UnboundedReceiver<Message>) {
    let addr = "127.0.0.1:1337";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Can't bind to address");

    let receiver = Arc::new(Mutex::new(receiver));
    let active_conn = Arc::new(Mutex::new(false));

    println!("Server started on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            stream,
            addr,
            active_conn.clone(),
            receiver.clone(),
        ));
    }
    return;
}
