mod error;
mod server;
mod state;

pub use error::ShortenError;
pub use server::{run, ShortenRequest, ShortenResponse};
pub use state::AppState;

lazy_static::lazy_static! {
    pub static ref LISTEN_ADDR: String = dotenvy::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8888".to_string());
}
