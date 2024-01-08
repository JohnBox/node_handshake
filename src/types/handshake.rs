use near_network_primitives::types::{PartialEdgeInfo, PeerChainInfoV2};
use near_primitives::block::GenesisId;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::hash::CryptoHash;
use near_primitives::network::PeerId;
use protobuf::MessageField;

use crate::proto;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Handshake {
    /// Current protocol version.
    pub protocol_version: u32,
    /// Oldest supported protocol version.
    pub oldest_supported_version: u32,
    /// Sender's peer id.
    pub sender_peer_id: PeerId,
    /// Receiver's peer id.
    pub target_peer_id: PeerId,
    /// Sender's listening addr.
    pub sender_listen_port: Option<u16>,
    /// Peer's chain information.
    pub sender_chain_info: PeerChainInfoV2,
    /// Represents new `edge`. Contains only `nonce` and `Signature` from the sender.
    pub partial_edge_info: PartialEdgeInfo,
}

impl From<Handshake> for proto::network::Handshake {
    fn from(value: Handshake) -> Self {
        proto::network::Handshake {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: MessageField::some(value.sender_peer_id.into()),
            target_peer_id: MessageField::some(value.target_peer_id.into()),
            sender_listen_port: value.sender_listen_port.map_or(0, u32::from),
            sender_chain_info: MessageField::some(value.sender_chain_info.into()),
            partial_edge_info: MessageField::some(value.partial_edge_info.into()),
            owned_account: MessageField::none(),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::Handshake> for Handshake {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: proto::network::Handshake) -> Result<Self, Self::Error> {
        Ok(Self {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: PeerId::try_from_slice(value.sender_peer_id.borsh.as_slice())?,
            target_peer_id: PeerId::try_from_slice(value.target_peer_id.borsh.as_slice())?,
            sender_listen_port: u16::try_from(value.sender_listen_port).map(|port| {
                if port == 0 {
                    None
                } else {
                    Some(port)
                }
            })?,
            sender_chain_info: PeerChainInfoV2 {
                genesis_id: GenesisId {
                    chain_id: value.sender_chain_info.genesis_id.chain_id.clone(),
                    hash: CryptoHash::try_from_slice(
                        value.sender_chain_info.genesis_id.hash.hash.as_slice(),
                    )?,
                },
                height: value.sender_chain_info.height,
                tracked_shards: value.sender_chain_info.tracked_shards.clone(),
                archival: value.sender_chain_info.archival,
            },
            partial_edge_info: PartialEdgeInfo::try_from_slice(
                value.partial_edge_info.borsh.as_slice(),
            )?,
        })
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

    #[test]
    fn test_serde() -> Result<()> {
        let genesis_id = GenesisId {
            chain_id: "testnet".to_string(),
            hash: CryptoHash::try_from_slice(
                ed25519_dalek::SecretKey::generate(&mut OsRng).as_bytes(),
            )?,
        };
        let sender_chain_info = PeerChainInfoV2 {
            genesis_id,
            height: 0,
            tracked_shards: vec![],
            archival: false,
        };
        let keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
        let sender_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            keypair.public.to_bytes(),
        )));
        let target_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            ed25519_dalek::Keypair::generate(&mut OsRng)
                .public
                .to_bytes(),
        )));
        let secret_key = SecretKey::ED25519(ED25519SecretKey(keypair.to_bytes()));
        let partial_edge_info =
            PartialEdgeInfo::new(&sender_peer_id, &target_peer_id, 1, &secret_key);
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 61,
            sender_peer_id,
            target_peer_id,
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
