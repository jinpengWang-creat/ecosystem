// Import necessary modules and libraries
use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{fmt::Display, str::FromStr};

// Define a constant key
const KEY: &[u8] = b"01234567890123456789012345678901";

// Define a User struct with various fields
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u8,
    #[serde(rename = "hobbies", skip_serializing_if = "Vec::is_empty", default)]
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[serde_as(as = "DisplayFromStr")]
    sensitive: Sensitive,
    #[serde_as(as = "DisplayFromStr")]
    url: http::Uri,
}

// Define a Sensitive struct
#[derive(Debug)]
struct Sensitive(String);

// Define an enum for work state
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
enum WorkState {
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminal,
}

// Main function
fn main() -> Result<()> {
    // Create a new user and serialize it to JSON
    let state = WorkState::OnLeave(Utc::now());
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        skills: vec![],
        state,
        data: vec![0, 1, 2, 3, 4, 5],
        sensitive: Sensitive::new("sensitive"),
        url: "https://example.com".parse()?,
    };
    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    // Deserialize the JSON back to a User
    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);
    Ok(())
}

// Function to encode data to base64
fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

// Function to decode base64 data
fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)
}

// Function to encrypt data
fn encode(data: &str) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, data.as_bytes())
        .map_err(|_e| anyhow::anyhow!("encryption failed"))?;

    let data = nonce.into_iter().chain(ciphertext).collect::<Vec<u8>>();
    Ok(URL_SAFE_NO_PAD.encode(data))
}

// Function to decrypt data
fn decode(data: &str) -> Result<String> {
    let encrypted_data = URL_SAFE_NO_PAD
        .decode(data.as_bytes())
        .map_err(|_e| anyhow::anyhow!("decryption failed"))?;
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let decrypted = cipher
        .decrypt(nonce, &encrypted_data[12..])
        .map_err(|_e| anyhow::anyhow!("decryption failed"))?;
    String::from_utf8(decrypted).map_err(|_e| anyhow::anyhow!("decryption failed"))
}

// Implement Display trait for Sensitive struct
impl Display for Sensitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = encode(&self.0).map_err(|_e| std::fmt::Error)?;
        write!(f, "{}", encoded)
    }
}

// Implement FromStr trait for Sensitive struct
impl FromStr for Sensitive {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = decode(s)?;
        Ok(Sensitive(decoded))
    }
}

// Implement a new method for Sensitive struct
impl Sensitive {
    pub fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}
