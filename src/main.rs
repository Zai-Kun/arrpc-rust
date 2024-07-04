mod consts;
mod handle_activity;
mod servers;
mod utils;

use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<Message>();

    let server1 = tokio::spawn(servers::web_side::start_server(rx));
    let server2 = tokio::spawn(servers::websockets::start_server(tx.clone()));
    let server3 = tokio::spawn(servers::ipc::start_server(tx.clone()));

    let (srv1_task, srv2_task, srv3_task) = tokio::join!(server1, server2, server3);
    srv1_task.unwrap();
    srv2_task.unwrap();
    srv3_task.unwrap();
}
