use std::error::Error;
use std::net::Ipv4Addr;
use std::sync::Arc;

use clap::Parser;
use near_network_primitives::types::RoutedMessageBody;
use tokio::net::{TcpListener, TcpStream};
use tokio::{join, pin};
use tokio::sync::oneshot::Sender;

use node_handshake::{ReceivePeerMessage, SendPeerMessage};
use node_handshake::config::Config;
use node_handshake::types::node::Node;
use node_handshake::types::peer_message::PeerMessage;

async fn listener(
    config: Config,
    listener_ready_notifier: Sender<()>,
    listener_node: Arc<Node>,
) {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let listener = TcpListener::bind((ip, config.sender_listen_port))
        .await.expect("port {config.sender_listen_port} already in use");

    listener_ready_notifier.send(()).unwrap();

    println!("Started listener on {} port", config.sender_listen_port);

    tokio::task::yield_now().await;

    loop {
        tokio::task::yield_now().await;
        match listener.accept().await {
            Ok((connection, from)) => {
                tokio::task::spawn({
                    let listener_node = listener_node.clone();
                    async move {
                        pin!(connection);
                        let peer_message = connection.as_mut().receive_peer_message().await.unwrap();

                        println!("<<< RECEIVE FROM {from:#?} HANDSHAKE {peer_message:#?}");
                        let PeerMessage::Tier2Handshake(ref target_handshake) = peer_message else { unreachable!("first message is Handshake") };


                        if listener_node.verify_handshake(target_handshake) {
                            println!("HANDSHAKE IS VALID");

                            let sender_handshake = listener_node.create_handshake(
                                target_handshake.sender_peer_id.clone(),
                                target_handshake.partial_edge_info.nonce,
                            );
                            let peer_message = PeerMessage::Tier2Handshake(sender_handshake);
                            println!(">>> SEND TO {from:?} HANDSHAKE {peer_message:#?}");

                            connection.as_mut().send_peer_message(peer_message).await.unwrap();

                            let peer_message = loop {
                                match connection.as_mut().receive_peer_message().await {
                                    Ok(peer_message) => break peer_message,
                                    Err(_) => continue
                                }
                            };

                            println!("<<< RECEIVE FROM {from:?} PING {peer_message:?}");

                            let PeerMessage::Routed(routed_message) = peer_message else { unreachable!("only accept Ping message") };
                            if routed_message.verify() {
                                println!("PING IS VALID");

                                let RoutedMessageBody::Ping(ref ping) = routed_message.msg.body else { unreachable!("only accept Ping message") };
                                let pong = listener_node.create_pong(routed_message.author.clone(), ping.nonce);
                                let peer_message = PeerMessage::Routed(pong.into());
                                println!(">>> SEND PONG TO PEER {peer_message:#?}");

                                connection.as_mut().send_peer_message(peer_message).await.unwrap();
                            } else {
                                println!("PING IS INVALID, CLOSE CONNECTION");
                                drop(connection);
                            };
                        } else {
                            println!("HANDSHAKE IS INVALID, CLOSE CONNECTION");
                            drop(connection);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("ERROR ACCEPT {e:?}");
                tokio::task::yield_now().await;
            }
        };
    }
}

async fn send_handshake(node: Arc<Node>, config: Config) {
    println!("Trying connect to {:?}", config.target_peer_info.addr);
    let connection = TcpStream::connect(config.target_peer_info.addr.unwrap()).await.unwrap();

    let handshake = node.create_handshake(
        config.target_peer_info.id.clone(),
        1,
    );
    let peer_message = PeerMessage::Tier2Handshake(handshake);
    println!(">>> OUTBOUND SEND HANDSHAKE");

    pin!(connection);

    connection.as_mut().send_peer_message(peer_message).await.unwrap();

    let _peer_message = connection.as_mut().receive_peer_message().await.unwrap();

    println!("<<< OUTBOUND RECEIVE HANDSHAKE");

    let ping = node.create_ping(config.target_peer_info.id.clone());
    let peer_message = PeerMessage::Routed(ping.into());
    println!(">>> OUTBOUND SEND PING");

    connection.as_mut().send_peer_message(peer_message).await.unwrap();

    let _peer_message = loop {
        match connection.as_mut().receive_peer_message().await {
            Ok(peer_message) => break peer_message,
            Err(_) => continue
        }
    };

    println!("<<< OUTBOUND RECEIVE PONG");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse();

    let sender_node = Arc::new(Node::from(config.clone()));
    println!("My node id {:}", sender_node.peer_id());

    let (listener_ready_notifier, listener_ready) = tokio::sync::oneshot::channel();

    let listener = {
        let listener_node = sender_node.clone();
        let config = config.clone();
        tokio::spawn(async move {
            listener(config, listener_ready_notifier, listener_node).await
        })
    };

    let sender = {
        let sender_node = sender_node.clone();
        let config = config.clone();
        tokio::spawn(async move {
            listener_ready.await.unwrap();
            send_handshake(sender_node, config).await
        })
    };

    join!(sender, listener);
    Ok(())
}