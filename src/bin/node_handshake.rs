use std::error::Error;
use clap::{Parser};
use tokio::net::TcpStream;
use tokio::pin;
use node_handshake::config::Config;
use node_handshake::{ReceivePeerMessage, SendPeerMessage};
use node_handshake::types::node::Node;
use node_handshake::types::peer_message::PeerMessage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();

    let connection = TcpStream::connect(config.target_peer_info.addr.unwrap()).await?;

    let node = Node::from(config);

    let handshake = node.create_handshake();
    let peer_message = PeerMessage::Tier2Handshake(handshake);
    println!(">>> SEND HANDSHAKE {peer_message:#?}");

    pin!(connection);

    connection.as_mut().send_peer_message(peer_message).await?;

    let peer_message = connection.as_mut().receive_peer_message().await?;

    println!("<<< RECEIVE HANDSHAKE {:#?}", peer_message);

    let ping = node.create_ping();
    let peer_message = PeerMessage::Routed(ping.into());
    println!(">>> SEND PING {peer_message:#?}");

    connection.as_mut().send_peer_message(peer_message).await?;

    let peer_message = loop {
        match connection.as_mut().receive_peer_message().await {
            Ok(peer_message) => break peer_message,
            Err(_) => continue
        }
    };

    println!("<<< RECEIVE PONG {peer_message:#?}");

    Ok(())
}