use std::str::FromStr;

use ed25519_dalek::Keypair;
use near_crypto::{ED25519PublicKey, ED25519SecretKey, PublicKey, SecretKey};
use near_network_primitives::time;
use near_network_primitives::types::{AccountOrPeerIdOrHash, PartialEdgeInfo, PeerChainInfoV2, PeerInfo, Ping, RawRoutedMessage, RoutedMessageBody, RoutedMessageV2};
use near_primitives::block::GenesisId;
use near_primitives::hash::CryptoHash;
use near_primitives::network::PeerId;
use rand::rngs::OsRng;
use crate::config::Config;

use crate::types::handshake::Handshake;

#[derive(Debug)]
pub struct Node {
    key_pair: Keypair,
    target_peer_info: PeerInfo,
    protocol_version: u32,
    oldest_supported_version: u32,
    sender_listen_port: Option<u16>,
}

impl From<Config> for Node {
    fn from(value: Config) -> Self {
        Self {
            key_pair: Keypair::generate(&mut OsRng),
            target_peer_info: value.target_peer_info,
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_listen_port: value.sender_listen_port.into(),
        }
    }
}

impl AsRef<Keypair> for Node {
    fn as_ref(&self) -> &Keypair {
        &self.key_pair
    }
}

impl Node {
    fn peer_id(&self) -> PeerId {
        PeerId::new(PublicKey::ED25519(ED25519PublicKey(self.as_ref().public.to_bytes())))
    }

    fn secret_key(&self) -> SecretKey {
        SecretKey::ED25519(ED25519SecretKey(self.as_ref().to_bytes()))
    }
    pub fn create_handshake(&self) -> Handshake {
        let sender_peer_id = self.peer_id();
        let sender_secret_key = self.secret_key();
        let target_peer_id = self.target_peer_info.id.clone();
        let genesis_hash: CryptoHash =
            CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap();
        let genesis_id: GenesisId = GenesisId {
            chain_id: "localnet".to_string(),
            hash: genesis_hash,
        };

        Handshake {
            protocol_version: self.protocol_version,
            oldest_supported_version: self.oldest_supported_version,
            sender_peer_id: sender_peer_id.clone(),
            target_peer_id: target_peer_id.clone(),
            sender_listen_port: self.sender_listen_port,
            sender_chain_info: PeerChainInfoV2 {
                genesis_id,
                height: 0,
                tracked_shards: vec![],
                archival: false,
            },
            partial_edge_info: PartialEdgeInfo::new(
                &sender_peer_id,
                &target_peer_id,
                1,
                &sender_secret_key,
            ),
        }
    }

    pub fn create_ping(&self) -> RoutedMessageV2 {
        let routed_message_body = RoutedMessageBody::Ping(Ping {
            nonce: 3,
            source: self.peer_id(),
        });

        let raw_routed_message = RawRoutedMessage {
            target: AccountOrPeerIdOrHash::PeerId(self.target_peer_info.id.clone()),
            body: routed_message_body,
        };

        let routed_message = raw_routed_message.sign(
            &self.secret_key(), 3, Some(time::Utc::now_utc()),
        );

        routed_message
    }
}


#[cfg(test)]
mod tests {
    use anyhow::Result;
    use near_crypto::{ED25519PublicKey, ED25519SecretKey, PublicKey, SecretKey};
    use near_network_primitives::types::{PartialEdgeInfo, PeerChainInfoV2};
    use near_primitives::block::GenesisId;
    use near_primitives::borsh::BorshDeserialize;
    use near_primitives::hash::CryptoHash;
    use near_primitives::network::PeerId;
    use rand::rngs::OsRng;

    use crate::proto::network;
    use crate::types::handshake::Handshake;
    use crate::types::node::Node;

    #[test]
    fn test_create_handshake() -> Result<()> {
        let genesis_id = GenesisId {
            chain_id: "testnet".to_string(),
            hash: CryptoHash::try_from_slice(
                ed25519_dalek::SecretKey::generate(&mut OsRng).as_bytes()
            )?,
        };
        let sender_chain_info = PeerChainInfoV2 {
            genesis_id,
            height: 0,
            tracked_shards: vec![],
            archival: false,
        };


        let (target_node, sender_node) = (Node::default(), Node::default());

        let partial_edge_info = PartialEdgeInfo::new(
            &PeerId::new(PublicKey::ED25519(ED25519PublicKey(sender_node.as_ref().public.to_bytes()))),
            &PeerId::new(PublicKey::ED25519(ED25519PublicKey(target_node.as_ref().public.to_bytes()))),
            1,
            &SecretKey::ED25519(ED25519SecretKey(sender_node.as_ref().to_bytes())),
        );
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 61,
            sender_peer_id: PeerId::new(PublicKey::ED25519(ED25519PublicKey(sender_node.as_ref().public.to_bytes()))),
            target_peer_id: PeerId::new(PublicKey::ED25519(ED25519PublicKey(target_node.as_ref().public.to_bytes()))),
            sender_listen_port: Some(51200),
            sender_chain_info,
            partial_edge_info,
        };


        let handshake_original = handshake.clone();
        let network_handshake: network::Handshake = handshake.into();
        let handshake_restored: Handshake = network_handshake.try_into().unwrap();
        assert_eq!(handshake_original, handshake_restored);

        Ok(())
    }
}
