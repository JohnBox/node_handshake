use near_crypto::{ED25519PublicKey, PublicKey};
use near_primitives::network::PeerId;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

mod proto;
mod types;


async fn sender_public_key() -> [u8; 32] {
    // let mut csprng = rand_core::OsRng::default();
    // let key_pair = SigningKey::generate(&mut csprng);
    // key_pair.verifying_key().to_bytes()
    [0; 32]
}

async fn target_public_key() -> [u8; 32] {
    const EC25519_PREFIX: &str = "ed25519:";
    let target_public_key = "ed25519:CCrQR8UzSNXahfK1USSrVsKWHgabo7jTwSjowgJUgMzF";
    let encoded_public_key = target_public_key.trim_start_matches(EC25519_PREFIX);
    let mut buffer = [0; 32];
    bs58::decode(encoded_public_key).onto(&mut buffer).unwrap();
    buffer
}

// async fn generate_handshake() -> Handshake {
//     let sender_public_key = PublicKey::ED25519(ED25519PublicKey::from(sender_public_key().await));
//     let target_public_key = PublicKey::ED25519(ED25519PublicKey::from(target_public_key().await));
//     let sender_peer_id = PeerId::new(sender_public_key);
//     let target_peer_id = PeerId::new(target_public_key);
//     let sender_chain_info = PeerChainInfoV2 {
//         genesis_id: Default::default(),
//         height: 0,
//         tracked_shards: vec![],
//         archival: false,
//     };
//     let partial_edge_info = PartialEdgeInfo { nonce: 1, signature: Default::default() };
//     let handshake = Handshake {
//         protocol_version: 63,
//         oldest_supported_version: 63,
//         sender_peer_id,
//         target_peer_id,
//         sender_listen_port: None,
//         sender_chain_info,
//         partial_edge_info,
//     };
//     handshake
// }

async fn connect_to_node() -> std::io::Result<TcpStream> {
    TcpStream::connect("127.0.0.1:24567").await
}

#[allow(dead_code)]
async fn send_handshake() {
    let _connection = connect_to_node().await.unwrap();
    // let handshake = generate_handshake().await;
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bytes::BytesMut;
    use ed25519_dalek::{Keypair, SecretKey, Signature, Signer};
    use near_primitives::block::GenesisId;
    use near_primitives::hash::CryptoHash;
    use protobuf::Message;
    use sha256::digest;
    use tokio::io::AsyncReadExt;

    use rand::rngs::OsRng;
    use crate::types::handshake::{Handshake, PeerMessage};

    use super::*;

    #[tokio::test]
    async fn test_connect_to_node() {
        assert!(connect_to_node().await.is_ok());
    }


    #[tokio::test]
    async fn test_create_public_key_for_node() {
        let public_key = sender_public_key().await;
        assert_eq!(public_key.len(), 32);
    }

    #[tokio::test]
    async fn test_decode_target_public_key() {
        let target_public_key = target_public_key().await;
        assert_eq!(target_public_key.len(), 32);
    }

    #[tokio::test]
    async fn test_send_handshake() {
        let mut connection = connect_to_node().await.unwrap();

        let mut csprng = OsRng::default();
        let sender_signature_key = Keypair::generate(&mut csprng);

        let seed = [114, 239, 96, 44, 49, 125, 81, 74, 83, 216, 75, 240, 26, 154, 153, 166, 102, 159, 217, 42, 105, 118, 226, 193, 190, 134, 116, 77, 237, 10, 51, 37];
        let secret_key = SecretKey::from_bytes(&seed).unwrap();
        let public_key: ed25519_dalek::PublicKey = (&secret_key).into();
        println!("{:?}", secret_key.as_bytes());
        println!("{:?}", public_key.as_bytes());
        let sp_key = [secret_key.to_bytes(), public_key.to_bytes()].concat();

        let sender_signature_key = Keypair::from_bytes(&sp_key).unwrap();
        // return;

        let sender_public_key_bytes = sender_signature_key.public.to_bytes();
        let sender_public_key = PublicKey::ED25519(ED25519PublicKey::from(
            sender_public_key_bytes
        ));
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
        println!("{:?}", edge_info);
        let sha256_edge_info = digest(edge_info);
        let sender_signature = sender_signature_key.sign(sha256_edge_info.as_bytes());

        let genesis_hash = CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap();
        let genesis_id = GenesisId {
            chain_id: "localnet".to_string(),
            hash: genesis_hash,
        };
        println!("My genesis id {:?}", genesis_id);
        let mut handshake = Handshake::default();
        handshake.target_peer_id = target_peer_id;
        handshake.sender_peer_id = sender_peer_id;
        handshake.sender_chain_info.genesis_id = genesis_id;

        handshake.partial_edge_info.signature = near_crypto::Signature::ED25519(sender_signature);

        let peer_message = PeerMessage::Tier2Handshake(handshake);
        println!("{:#?}", peer_message);

        let network_peer_message: proto::network::PeerMessage = peer_message.into();

        let message = network_peer_message.write_to_bytes().unwrap();
        let message_size = message.len();

        let result = connection.write_u32_le(message_size as u32).await;
        println!("{:?}", result);
        let result = connection.write_all(&message).await;
        println!("{:?}", result);
        let result = connection.flush().await;
        println!("{:?}", result);

        let mut buf = BytesMut::with_capacity(8_196);
        let result = connection.read_buf(&mut buf).await;
        println!("{:?}", result);
        println!("{:?}", String::from_utf8(buf.to_vec()));
    }
}