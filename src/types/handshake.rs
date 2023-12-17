use near_crypto::{KeyType, PublicKey, Signature};
use near_network_primitives::types::{PartialEdgeInfo, PeerChainInfoV2};
use near_primitives::block::GenesisId;
use near_primitives::borsh::BorshSerialize;
use near_primitives::hash::CryptoHash;
use near_primitives::network::PeerId;
use protobuf::MessageField;
use crate::proto;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PeerMessage {
    Tier1Handshake(Handshake),
    Tier2Handshake(Handshake),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Handshake {
    /// Current protocol version.
    pub(crate) protocol_version: u32,
    /// Oldest supported protocol version.
    pub(crate) oldest_supported_version: u32,
    /// Sender's peer id.
    pub(crate) sender_peer_id: PeerId,
    /// Receiver's peer id.
    pub(crate) target_peer_id: PeerId,
    /// Sender's listening addr.
    pub(crate) sender_listen_port: Option<u16>,
    /// Peer's chain information.
    pub(crate) sender_chain_info: PeerChainInfoV2,
    /// Represents new `edge`. Contains only `nonce` and `Signature` from the sender.
    pub(crate) partial_edge_info: PartialEdgeInfo,
}

impl From<PublicKey> for proto::network::PublicKey {
    fn from(value: PublicKey) -> Self {
        proto::network::PublicKey {
            borsh: value.try_to_vec().unwrap(),
            ..Default::default()
        }
    }
}

impl From<PeerId> for proto::network::PublicKey {
    fn from(value: PeerId) -> Self {
        value.public_key().clone().into()
    }
}


impl From<CryptoHash> for proto::network::CryptoHash {
    fn from(value: CryptoHash) -> Self {
        proto::network::CryptoHash {
            hash: value.0.into(),
            ..Default::default()
        }
    }
}

impl From<GenesisId> for proto::network::GenesisId {
    fn from(value: GenesisId) -> Self {
        proto::network::GenesisId {
            chain_id: value.chain_id,
            hash: MessageField::some(value.hash.into()),
            ..Default::default()
        }
    }
}

impl From<PeerChainInfoV2> for proto::network::PeerChainInfo {
    fn from(value: PeerChainInfoV2) -> Self {
        proto::network::PeerChainInfo {
            genesis_id: MessageField::some(value.genesis_id.into()),
            height: value.height,
            tracked_shards: value.tracked_shards,
            archival: value.archival,
            ..Default::default()
        }
    }
}

impl From<PartialEdgeInfo> for proto::network::PartialEdgeInfo {
    fn from(value: PartialEdgeInfo) -> Self {
        proto::network::PartialEdgeInfo {
            borsh: value.try_to_vec().unwrap(),
            ..Default::default()
        }
    }
}

impl From<Handshake> for proto::network::Handshake {
    fn from(value: Handshake) -> Self {
        proto::network::Handshake {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: MessageField::some(value.sender_peer_id.into()),
            target_peer_id: MessageField::some(value.target_peer_id.into()),
            sender_listen_port: value.sender_listen_port.map_or(0,|port| port as u32),
            sender_chain_info: MessageField::some(value.sender_chain_info.into()),
            partial_edge_info: MessageField::some(value.partial_edge_info.into()),
            owned_account: MessageField::none(),
            ..Default::default()
        }
    }
}

impl From<PeerMessage> for proto::network::PeerMessage {
    fn from(value: PeerMessage) -> Self {
        proto::network::PeerMessage {
            message_type: Some(match value {
                PeerMessage::Tier1Handshake(handshake) =>
                    proto::network::peer_message::Message_type::Tier1Handshake(handshake.into()),
                PeerMessage::Tier2Handshake(handshake) =>
                    proto::network::peer_message::Message_type::Tier2Handshake(handshake.into()),
            }),
            ..Default::default()
        }
    }
}

impl Default for Handshake {
    fn default() -> Self {
        let sender_peer_id = PeerId::random();
        let target_peer_id = PeerId::random();
        let genesis_hash = "GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL".parse().unwrap();
        let genesis_id = GenesisId {
            chain_id: "localnet".to_string(),
            hash: genesis_hash,
        };
        let sender_chain_info = PeerChainInfoV2 {
            genesis_id,
            height: 0,
            tracked_shards: vec![],
            archival: false,
        };
        let edge_signature = Signature::empty(KeyType::ED25519);
        let partial_edge_info = PartialEdgeInfo {
            nonce: 1,
            signature: edge_signature,
        };
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 0,
            sender_peer_id,
            target_peer_id,
            sender_listen_port: Some(12345),
            sender_chain_info,
            partial_edge_info,
        };


        handshake
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use near_crypto::{KeyType, Signature};
    use near_network_primitives::types::{PartialEdgeInfo, PeerChainInfoV2};
    use near_primitives::block::GenesisId;
    use near_primitives::hash::CryptoHash;
    use near_primitives::network::PeerId;

    use crate::types::handshake::Handshake;

    #[test]
    fn test_handshake() {
        let sender_peer_id = PeerId::random();
        let target_peer_id = PeerId::random();
        let genesis_hash = CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap();
        let genesis_id = GenesisId {
            chain_id: "localnet".to_string(),
            hash: genesis_hash,
        };
        println!("My genesis id {:?}", genesis_id);
        let sender_chain_info = PeerChainInfoV2 {
            genesis_id,
            height: 0,
            tracked_shards: vec![],
            archival: false,
        };
        let edge_signature = Signature::empty(KeyType::ED25519);
        let partial_edge_info = PartialEdgeInfo {
            nonce: 0,
            signature: edge_signature,
        };
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 63,
            sender_peer_id,
            target_peer_id,
            sender_listen_port: None,
            sender_chain_info,
            partial_edge_info,
        };

        println!("{:#?}", handshake);

        let network_handshake: crate::proto::network::Handshake = handshake.into();

        println!("{:#?}", network_handshake);
    }
}
