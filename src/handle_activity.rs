use std::{collections::HashMap, i32};

use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use super::utils;
use chrono::Utc;
use tokio::sync::mpsc::UnboundedSender;

pub fn clear_activity(pid: i32, tx: UnboundedSender<Message>) {
    let msg = Message::text(format!(
        r#"{{"activity": null, "pid": {}, "socketId": "0"}}"#,
        pid
    ));
    tx.send(msg).expect("Error sending to channel");
}

pub fn handle(activity: Value, client_id: &str, tx: UnboundedSender<Message>) -> Option<i32> {
    let (pid, activity) = if let Some(some) = activity.get("args") {
        if let Some(some) = utils::deserialize_args(some) {
            some
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mut activity = if let Some(some) = activity {
        some
    } else {
        clear_activity(pid, tx);
        return Some(pid);
    };

    let mut metadata = HashMap::new();
    let mut extra_buttons: Option<Vec<String>> = None;

    if let Some(buttons) = &activity.buttons {
        metadata.insert(
            "button_urls".to_string(),
            Some(buttons.iter().map(|x| x.url.clone()).collect::<Vec<_>>()),
        );
        extra_buttons = Some(buttons.iter().map(|x| x.label.clone()).collect::<Vec<_>>());
    }

    if let Some(timestamps) = activity.timestamps.as_mut() {
        for (_key, value) in timestamps.iter_mut() {
            if Utc::now().timestamp_millis().to_string().len() as i64
                - value.to_string().len() as i64
                > 2
            {
                *value *= 1000;
            }
        }
    }

    let value = serde_json::json!({
        "activity": {
            "application_id": client_id,
            "type": activity.r#type,
            "metadata": metadata,
            "flags": 0,
            "assets": activity.assets,
            "buttons": extra_buttons,
            "state": activity.state,
            "timestamps": activity.timestamps,
            "details": activity.details
        },
        "pid": pid,
        "socketId": "0"
    });

    tx.send(Message::text(serde_json::to_string(&value).unwrap()))
        .unwrap();
    return Some(pid);
}
