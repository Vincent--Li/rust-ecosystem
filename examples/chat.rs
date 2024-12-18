use std::{fmt::Display, net::SocketAddr, sync::Arc};

use anyhow::Result;
use dashmap::DashMap;
use derive_more::derive::Debug;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt as _, Layer as _,
};

const MAX_MESSAGES: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug, Clone)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "127.0.0.1:8081";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    let state = Arc::new(State::default());

    loop {
        let (mut stream, addr) = listener.accept().await?;
        info!("accepted from {}", addr);
        let state_cloned = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_cloned, addr, stream).await {
                warn!("failed to handle peer: {}", e);
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    // frame 工具, 将字节流按Lines 分隔符进行解析
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username, stream).await;

    state.broadcast(addr, Arc::new(Message::user_joined(&peer.username))).await;
    info!("{} joined the chat", peer.username);

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("failed to read line from stream: {}", e);
                break;
            }
        };

        let message = Arc::new( Message::chat(&peer.username, &line));

        state.broadcast(addr, message.clone()).await;
    }

    state.peers.remove(&addr);

    let message = Arc::new(Message::user_left(&peer.username));
    state.broadcast(addr, message).await;
    info!("{} left the chat", peer.username);

    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(Arc::clone(&message)).await {
                warn!("failed to send message to {}: {}", peer.key(), e);
                // if send failed, peer might be gone, remove peer from state
                self.peers.remove(peer.key());
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();

        // recieve messages from the peer and broadcast them to all other peers
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("failed to send message to {}: {}", addr, e);
                }
            }
        });

        // return peer
        Peer {
            username,
            stream: stream_receiver,
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

    // impl Into<String> 更广泛的接收可转换成Into类型的参数
    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        let content = format!("{}", content.into());
        Self::Chat {
            sender: sender.into(),
            content,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
          Message::UserJoined(content) => write!(f, "{}", content),
          Message::UserLeft(content) => write!(f, "{}", content),
          Message::Chat { sender, content } => write!(f, "{}: {}", sender, content),
      }
    }
}

