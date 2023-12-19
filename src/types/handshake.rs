use near_crypto::{KeyType, Signature};
use near_network_primitives::time;
use near_network_primitives::types::{PartialEdgeInfo, PeerChainInfoV2};
use near_primitives::block::GenesisId;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::network::PeerId;
use protobuf::MessageField;
use protobuf::well_known_types::timestamp::Timestamp;

use crate::proto;

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

impl Default for Handshake {
    fn default() -> Self {
        let sender_peer_id = PeerId::random();
        let target_peer_id = PeerId::random();
        let genesis_hash = "GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL"
            .parse()
            .unwrap();
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum HandshakeFailureReason {
    ProtocolVersionMismatch {
        version: u32,
        oldest_supported_version: u32,
    },
    GenesisMismatch(GenesisId),
    InvalidTarget,
}

impl From<Handshake> for proto::network::Handshake {
    fn from(value: Handshake) -> Self {
        proto::network::Handshake {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: MessageField::some(value.sender_peer_id.into()),
            target_peer_id: MessageField::some(value.target_peer_id.into()),
            sender_listen_port: value.sender_listen_port.map_or(0, |port| port as u32),
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
        let sender_chain_info = PeerChainInfoV2::default();
        let partial_edge_info = PartialEdgeInfo::default();
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
            sender_chain_info,
            partial_edge_info,
        })
    }
}

pub fn utc_to_proto(x: &time::Utc) -> Timestamp {
    Timestamp {
        seconds: x.unix_timestamp(),
        // x.nanosecond() is guaranteed to be in range [0,10^9).
        nanos: x.nanosecond() as i32,
        ..Default::default()
    }
}

// pub fn utc_from_proto(x: &Timestamp) -> Result<time::Utc, ParseTimestampError> {
//     time::Utc::from_unix_timestamp_nanos((x.seconds as i128 * 1_000_000_000) + (x.nanos as i128))
// }
