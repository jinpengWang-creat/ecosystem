use anyhow::Result;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u8,
    #[serde(rename = "hobbies")]
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

enum WorkState {
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminal,
}

fn main() -> Result<()> {
    let state = WorkState::OnLeave(Utc::now());
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        skills: vec!["Rust".to_string(), "TypeScript".to_string()],
        state,
        data: vec![0, 1, 2, 3, 4, 5],
    };
    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);
    Ok(())
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)
}
