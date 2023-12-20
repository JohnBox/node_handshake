use near_crypto::{ED25519PublicKey, PublicKey};
use near_primitives::network::PeerId;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

mod proto;
mod types;


async fn target_public_key() -> [u8; 32] {
    const EC25519_PREFIX: &str = "ed25519:";
    let target_public_key = "ed25519:CCrQR8UzSNXahfK1USSrVsKWHgabo7jTwSjowgJUgMzF";
    let encoded_public_key = target_public_key.trim_start_matches(EC25519_PREFIX);
    let mut buffer = [0; 32];
    bs58::decode(encoded_public_key).onto(&mut buffer).unwrap();
    buffer
}

async fn connect_to_node() -> std::io::Result<TcpStream> {
    TcpStream::connect("127.0.0.1:24567").await
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bytes::BytesMut;
    use ed25519_dalek::{Keypair, SecretKey, Signer};
    use near_crypto::ED25519SecretKey;
    use near_network_primitives::time;
    use near_network_primitives::types::{AccountOrPeerIdOrHash, PartialEdgeInfo, PeerChainInfoV2, Ping, RawRoutedMessage, RoutedMessageBody};
    use near_primitives::block::GenesisId;
    use near_primitives::hash::CryptoHash;
    use protobuf::Message;
    use tokio::io::AsyncReadExt;

    use crate::types::handshake::Handshake;
    use crate::types::peer_message::PeerMessage;

    use super::*;

    async fn test_send_handshake() {
        let mut connection = connect_to_node().await.unwrap();

        // let mut csprng = OsRng::default();

        let seed = [
            114, 239, 96, 44, 49, 125, 81, 74, 83, 216, 75, 240, 26, 154, 153, 166, 102, 159, 217,
            42, 105, 118, 226, 193, 190, 134, 116, 77, 237, 10, 51, 37,
        ];
        let secret_key = SecretKey::from_bytes(&seed).unwrap();
        let public_key: ed25519_dalek::PublicKey = (&secret_key).into();
        let sp_key = [secret_key.to_bytes(), public_key.to_bytes()].concat();

        let sender_signature_key = Keypair::from_bytes(&sp_key).unwrap();

        let sender_public_key_bytes = sender_signature_key.public.to_bytes();
        let sender_public_key = PublicKey::ED25519(ED25519PublicKey::from(sender_public_key_bytes));
        let sender_peer_id = PeerId::new(sender_public_key);

        let target_public_key_bytes = target_public_key().await;
        let target_public_key = PublicKey::ED25519(ED25519PublicKey::from(target_public_key_bytes));
        let target_peer_id = PeerId::new(target_public_key);

        let mut edge_info = vec![];
        edge_info.push(0);
        if target_public_key_bytes < sender_public_key_bytes {
            edge_info.extend(target_public_key_bytes);
        } else {
            edge_info.extend(sender_public_key_bytes);
        }
        edge_info.push(0);
        if target_public_key_bytes < sender_public_key_bytes {
            edge_info.extend(sender_public_key_bytes);
        } else {
            edge_info.extend(target_public_key_bytes);
        }
        edge_info.push(1);
        edge_info.extend([0; 7]);

        let sha256_edge_info = sha256::digest(edge_info);
        let sha256_edge_info = hex::decode(sha256_edge_info).unwrap();
        let sender_signature = sender_signature_key.sign(&sha256_edge_info);

        let genesis_hash =
            CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap();
        let genesis_id = GenesisId {
            chain_id: "localnet".to_string(),
            hash: genesis_hash,
        };
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 42,
            sender_peer_id: sender_peer_id.clone(),
            target_peer_id: target_peer_id.clone(),
            sender_listen_port: Some(12345),
            sender_chain_info: PeerChainInfoV2 {
                genesis_id,
                height: 0,
                tracked_shards: vec![],
                archival: false,
            },
            partial_edge_info: PartialEdgeInfo {
                nonce: 1,
                signature: near_crypto::Signature::ED25519(sender_signature),
            },
        };
        let peer_message = PeerMessage::Tier2Handshake(handshake);

        let network_peer_message: proto::network::PeerMessage = peer_message.into();

        let message = network_peer_message.write_to_bytes().unwrap();
        let message_size = message.len();

        let _result = connection.write_u32_le(message_size as u32).await;
        let _result = connection.write_all(&message).await;
        let _result = connection.flush().await;

        let mut buf = BytesMut::with_capacity(8_196);
        println!("{:X}", buf);
        let message_len = connection.read_u32_le().await.unwrap();
        println!("MESSAGE LENGTH = {message_len}");
        connection.read_buf(&mut buf).await.unwrap();
        let message = proto::network::PeerMessage::parse_from_bytes(
            &buf[..message_len as usize]
        ).unwrap();

        println!("NETWORK PEER MESSAGE {:?}", message);

        let peer_message: Result<PeerMessage, _> = message.try_into();

        if peer_message.is_err() {
            println!("ERROR {:?}", peer_message.unwrap_err());
        } else {
            println!("PEER MESSAGE {:#?}", peer_message.unwrap());
        }

        // Ping

        let routed_message_body = RoutedMessageBody::Ping(Ping {
            nonce: 3,
            source: sender_peer_id.clone(),
        });


        let raw_routed_message = RawRoutedMessage {
            target: AccountOrPeerIdOrHash::PeerId(target_peer_id.clone()),
            body: routed_message_body.clone(),
        };

        let secret_key: near_crypto::SecretKey = near_crypto::SecretKey::ED25519(
            ED25519SecretKey(sender_signature_key.to_bytes())
        );
        let routed_message = raw_routed_message.sign(
            &secret_key, 3, Some(time::Utc::now_utc()),
        );

        let ping = PeerMessage::Routed(Box::new(routed_message));
        println!("{:#?}", ping);

        let network_ping: proto::network::PeerMessage = ping.into();

        println!("{:?}", network_ping);

        let message = network_ping.write_to_bytes().unwrap();
        let message_size = message.len();

        let result = connection.write_u32_le(message_size as u32).await;
        println!("{:?}", result);
        let result = connection.write_all(&message).await;
        println!("{:?}", result);
        let result = connection.flush().await;
        println!("{:?}", result);
        let mut retries = 0;
        loop {
            let mut buf = BytesMut::with_capacity(8_196);
            println!("{:X}", buf);
            let message_len = connection.read_u32_le().await.unwrap();
            println!("MESSAGE LENGTH = {message_len}");
            connection.read_buf(&mut buf).await.unwrap();
            let message = proto::network::PeerMessage::parse_from_bytes(
                &buf[..message_len as usize]
            ).unwrap();

            println!("NETWORK PEER MESSAGE {:?}", message);

            let peer_message: Result<PeerMessage, _> = message.try_into();

            if peer_message.is_err() {
                println!("ERROR {:?}", peer_message.unwrap_err());
                retries += 1;
            } else {
                println!("PEER MESSAGE {:#?}", peer_message.unwrap());
            }

            if retries > 1 {
                break;
            }
        }
    }
}
