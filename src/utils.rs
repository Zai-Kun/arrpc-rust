use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Args {
    activity: Option<Activity>,
    pid: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Activity {
    pub assets: Option<HashMap<String, String>>,
    pub buttons: Option<Vec<Button>>,
    pub state: Option<String>,
    pub timestamps: Option<HashMap<String, i64>>,
    pub details: Option<String>,
    #[serde(default = "default_type")]
    pub r#type: u8,
}

#[derive(Deserialize, Serialize)]
pub struct Button {
    pub label: String,
    pub url: String,
}

fn default_type() -> u8 {
    0
}

pub fn deserialize_args(args: &Value) -> Option<(i32, Option<Activity>)> {
    let deserialized_args: Args = if let Ok(ok) = serde_json::from_value(args.clone()) {
        ok
    } else {
        return None;
    };
    Some((deserialized_args.pid, deserialized_args.activity))
}
