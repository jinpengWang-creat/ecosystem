use std::{fmt::Display, net::SocketAddr, sync::Arc};

use anyhow::Result;
use cfg_if::cfg_if;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[derive(Debug)]
struct Peer {
    username: String,
    receiver: SplitStream<Framed<TcpStream, WindowsLinesCodec>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // init tracing
    let layer = fmt::layer().pretty().with_filter(LevelFilter::INFO);
    let console_layer = console_subscriber::spawn();
    tracing_subscriber::registry()
        .with(layer)
        .with(console_layer)
        .init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on: {}", addr);
    let state = Arc::new(State::default());
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("accepted connection from: {:?}", addr);
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, state).await {
                info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

async fn handle_client(
    stream: tokio::net::TcpStream,
    addr: SocketAddr,
    state: Arc<State>,
) -> Result<()> {
    let codec = WindowsLinesCodec::new();

    let mut framed_stream = Framed::new(stream, codec);

    framed_stream
        .send("Please enter your username:".to_string())
        .await?;

    let username = match framed_stream.next().await {
        Some(Ok(username)) => username,
        _ => return Ok(()),
    };

    let mut peer = state.add(addr, username, framed_stream);
    let message = Arc::new(Message::user_joined(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    while let Some(message) = peer.receiver.next().await {
        let message = match message {
            Ok(message) => message,
            Err(e) => {
                warn!("Failed to read message from {}: {:?}", addr, e);
                break;
            }
        };

        info!("{}: {}", peer.username, message);
        if message.is_empty() {
            continue;
        }

        if message == "/quit" {
            break;
        }

        state
            .broadcast(
                addr,
                Arc::new(Message::Chat {
                    sender: peer.username.clone(),
                    content: message,
                }),
            )
            .await;
    }

    state.peers.remove(&addr);
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    Ok(())
}

impl State {
    fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, WindowsLinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(32);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {:?}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            receiver: stream_receiver,
        }
    }

    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(Arc::clone(&message)).await {
                warn!("Failed to send message to {}: {:?}", peer.key(), e);
            }
        }
    }
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} joined the chat", username);
        Self::UserJoined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("{} left the chat", username);
        Self::UserLeft(content)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserJoined(content) => write!(f, "{}", content),
            Self::UserLeft(content) => write!(f, "{}", content),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use bytes::BytesMut;
        use tokio_util::codec::{Decoder, Encoder};

        #[derive(Debug)]
        struct WindowsLinesCodec {
            inner: LinesCodec,
        }

        impl WindowsLinesCodec {

            fn new() -> Self {
                info!("Using WindowsLinesCodec");
                Self {
                    inner: LinesCodec::new(),
                }
            }
        }

        impl Decoder for WindowsLinesCodec {
            type Item = String;
            type Error = <LinesCodec as Decoder>::Error;

            fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
                if let Some(mut line) = self.inner.decode(src)? {
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    Ok(Some(line))
                } else {
                    Ok(None)
                }
            }
        }

        impl Encoder<String> for WindowsLinesCodec {
            type Error = <LinesCodec as Encoder<String>>::Error;

            fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
                self.inner.encode(item + "\r", dst)
            }
        }
    } else {
        type WindowsLinesCodec = LinesCodec;
    }
}
