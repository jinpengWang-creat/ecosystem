#![allow(unused)]
use std::{fmt::Display, net::SocketAddr};

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[derive(Debug, Clone)]
struct ChatState {
    peers: DashMap<SocketAddr, ()>,
}

#[derive(Debug)]
enum ChatMessage {
    UserJoin(UserJoin),
    UserLeft(UserLeft),
    Chat(Chat),
}
#[derive(Debug)]
struct UserJoin {
    username: String,
    join_time: DateTime<Utc>,
}

impl UserJoin {
    fn new(username: impl Into<String>) -> Self {
        UserJoin {
            username: username.into(),
            join_time: Utc::now(),
        }
    }
}

#[derive(Debug)]
struct UserLeft {
    username: String,
    left_time: DateTime<Utc>,
}

impl UserLeft {
    fn new(username: impl Into<String>) -> Self {
        UserLeft {
            username: username.into(),
            left_time: Utc::now(),
        }
    }
}

#[derive(Debug)]
struct Chat {
    sender: String,
    msg: String,
    send_time: DateTime<Utc>,
}

impl Chat {
    fn new(sender: impl Into<String>, msg: impl Into<String>) -> Self {
        Chat {
            sender: sender.into(),
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
            }) => write!(f, "({})[{} join the chat!] ", join_time, username),
            ChatMessage::UserLeft(UserLeft {
                username,
                left_time,
            }) => write!(f, "({})[{} join the chat!] ", left_time, username),
            ChatMessage::Chat(Chat {
                sender,
                msg,
                send_time: _,
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

    // accecpt connection
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accept connection from {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr).await {
                warn!("Unexpected error: {:?}", e);
            }
        });
    }
}

async fn handle_client(stream: TcpStream, _addr: SocketAddr) -> Result<()> {
    let mut framed = Framed::new(stream, ChatLineCodec::new());
    let username = get_username(&mut framed).await?;
    println!("get username: {}", username);
    Ok(())
}

async fn get_username(framed: &mut Framed<TcpStream, ChatLineCodec>) -> Result<String> {
    framed.send("Please enter your username:").await?;
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
    if #[cfg(target_os = "window")] {
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
                self.inner.encode(format!("{}{}", item, "\r\n"), dst)
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
