#![allow(unused)]
use std::{fmt::Display, net::SocketAddr, sync::Arc};

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_util::codec::Framed;
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[derive(Debug)]
struct ChatStateBackend {
    peers: DashMap<SocketAddr, mpsc::Sender<ChatMessage>>,
    message_receiver: mpsc::Receiver<ChatMessage>,
}

impl ChatStateBackend {
    fn new(message_receiver: Receiver<ChatMessage>) -> Self {
        ChatStateBackend {
            peers: DashMap::new(),
            message_receiver,
        }
    }

    fn run(mut self) {
        tokio::spawn(async move {
            while let Some(msg) = self.message_receiver.recv().await {
                let send_msg = msg.clone();
                match msg {
                    ChatMessage::UserJoin(UserJoin {
                        username,
                        join_time,
                        message_sender,
                        join_addr,
                    }) => {
                        self.peers.insert(join_addr, message_sender.clone());
                        info!("{} join the chat!", username);
                        self.broadcast(join_addr, send_msg.clone()).await;
                    }
                    ChatMessage::UserLeft(UserLeft {
                        username,
                        left_time,
                        left_addr,
                    }) => {
                        self.peers.remove(&left_addr);
                        info!("{} left the chat!", username);
                        self.broadcast(left_addr, send_msg.clone()).await;
                    }
                    ChatMessage::Chat(Chat {
                        sender,
                        msg: chat_msg,
                        send_time,
                        sender_addr,
                    }) => {
                        info!("[{}]: {}", sender, chat_msg);
                        self.broadcast(sender_addr, send_msg.clone()).await;
                    }
                }
            }
        });
    }

    async fn broadcast(&self, exclude_addr: SocketAddr, msg: ChatMessage) {
        for peer in self.peers.iter() {
            if peer.key() == &exclude_addr {
                continue;
            }
            info!("broadcast {:?} to {:?}", msg, peer.key());
            let _ = peer.value().send(msg.clone()).await;
        }
    }
}

#[derive(Debug, Clone)]
enum ChatMessage {
    UserJoin(UserJoin),
    UserLeft(UserLeft),
    Chat(Chat),
}
#[derive(Debug, Clone)]
struct UserJoin {
    username: String,
    join_time: DateTime<Utc>,
    message_sender: mpsc::Sender<ChatMessage>,
    join_addr: SocketAddr,
}

impl UserJoin {
    fn new(
        username: impl Into<String>,
        message_sender: Sender<ChatMessage>,
        join_addr: SocketAddr,
    ) -> Self {
        UserJoin {
            username: username.into(),
            join_time: Utc::now(),
            message_sender,
            join_addr,
        }
    }
}

#[derive(Debug, Clone)]
struct UserLeft {
    username: String,
    left_time: DateTime<Utc>,
    left_addr: SocketAddr,
}

impl UserLeft {
    fn new(username: impl Into<String>, left_addr: SocketAddr) -> Self {
        UserLeft {
            username: username.into(),
            left_time: Utc::now(),
            left_addr,
        }
    }
}

#[derive(Debug, Clone)]
struct Chat {
    sender: String,
    sender_addr: SocketAddr,
    msg: String,
    send_time: DateTime<Utc>,
}

impl Chat {
    fn new(sender: impl Into<String>, msg: impl Into<String>, sender_addr: SocketAddr) -> Self {
        Chat {
            sender: sender.into(),
            sender_addr,
            msg: msg.into(),
            send_time: Utc::now(),
        }
    }
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatMessage::UserJoin(UserJoin {
                username,
                join_time,
                message_sender: _,
                join_addr: _,
            }) => write!(f, "({})[{} join the chat!] ", join_time, username),
            ChatMessage::UserLeft(UserLeft {
                username,
                left_time,
                left_addr: _,
            }) => write!(f, "({})[{} left the chat!] ", left_time, username),
            ChatMessage::Chat(Chat {
                sender,
                msg,
                send_time: _,
                sender_addr: _,
            }) => write!(f, "[{}]: {}", sender, msg),
        }
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    // init tracing
    let layer = fmt::layer()
        .pretty()
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // create tcp listener
    let addr = "0.0.0.0:8888";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    // create chat state backend
    let (message_sender, message_receiver) = mpsc::channel(32);
    let chat_state = ChatStateBackend::new(message_receiver);
    chat_state.run();

    // accept connection
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accept connection from {:?}", addr);
        let message_sender = message_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, message_sender).await {
                warn!("Unexpected error: {:?}", e);
            }
        });
    }
}

#[instrument]
async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    message_sender: mpsc::Sender<ChatMessage>,
) -> Result<()> {
    let mut framed = Framed::new(stream, ChatLineCodec::new());
    let username = get_username(&mut framed).await?;

    // user join
    let (tx, mut rx) = mpsc::channel(32);
    let user_join = UserJoin::new(&username, tx, addr);
    message_sender
        .send(ChatMessage::UserJoin(user_join))
        .await?;

    let (mut stream_sender, stream_receiver) = framed.split();
    start_peer_receiver(username, addr, stream_receiver, message_sender);
    start_peer_sender(rx, stream_sender);
    Ok(())
}

fn start_peer_receiver(
    username: String,
    addr: SocketAddr,
    mut message_receiver: SplitStream<Framed<TcpStream, ChatLineCodec>>,
    message_sender: Sender<ChatMessage>,
) {
    tokio::spawn(async move {
        while let Some(msg_ret) = message_receiver.next().await {
            if let Ok(msg) = msg_ret {
                if msg.eq("/quit") {
                    break;
                }
                let msg = Chat::new(&username, msg, addr);
                if let Err(e) = message_sender.send(ChatMessage::Chat(msg)).await {
                    warn!("send chat message failed: {:?}", e);
                }
            } else {
                warn!("receive chat message failed: {:?}", msg_ret);
            }
        }
        let user_left = UserLeft::new(username, addr);
        if let Err(e) = message_sender.send(ChatMessage::UserLeft(user_left)).await {
            warn!("send user left message failed: {:?}", e);
        }
    });
}

fn start_peer_sender(
    mut message_receiver: mpsc::Receiver<ChatMessage>,
    mut message_sender: SplitSink<Framed<TcpStream, ChatLineCodec>, String>,
) {
    tokio::spawn(async move {
        while let Some(msg) = message_receiver.recv().await {
            let msg = format!("{}", msg);
            message_sender.send(msg).await;
        }
    });
}

async fn get_username(framed: &mut Framed<TcpStream, ChatLineCodec>) -> Result<String> {
    framed
        .send("Please enter your username:".to_string())
        .await?;
    let username = if let Some(name) = framed.next().await {
        match name {
            Ok(username) => username,
            Err(e) => return Err(anyhow::anyhow!("lines codes error: {:?}", e)),
        }
    } else {
        warn!("get none when receive username!");
        return Ok("unknown".to_string());
    };
    Ok(username)
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use tokio_util::codec::{Decoder, Encoder, LinesCodec, LinesCodecError};
        #[derive(Debug)]
        struct ChatLineCodec {
            inner: LinesCodec,
        }

        impl Decoder for ChatLineCodec {
            type Item = String;
            type Error = LinesCodecError;

            fn decode(
                &mut self,
                src: &mut bytes::BytesMut,
            ) -> std::prelude::v1::Result<Option<Self::Item>, Self::Error> {
                let ret = self.inner.decode(src)?;
                let ret = ret.map(|mut val| {
                    if val.ends_with('\r') {
                        val.pop();
                    }
                    val
                });
                Ok(ret)
            }
        }

        impl Encoder<String> for ChatLineCodec {
            type Error = LinesCodecError;

            fn encode(
                &mut self,
                item: String,
                dst: &mut bytes::BytesMut,
            ) -> std::prelude::v1::Result<(), Self::Error> {
                self.inner.encode(item + "\r", dst)
            }
        }

        impl ChatLineCodec {
            pub fn new() -> Self {
                ChatLineCodec {
                    inner: LinesCodec::new(),
                }
            }
        }


    } else {
        use tokio_util::codec::{ LinesCodec};
        type ChatLineCodec = LinesCodec;
    }
}
