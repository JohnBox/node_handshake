use near_network_primitives::time;
use near_network_primitives::time::Utc;
use near_network_primitives::types::{RoutedMessage, RoutedMessageV2};
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};
use protobuf::MessageField;
use protobuf::well_known_types::timestamp::Timestamp;

use crate::proto;
use crate::proto::network::peer_message::Message_type;
use crate::types::handshake::Handshake;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PeerMessage {
    Tier1Handshake(Handshake),
    Tier2Handshake(Handshake),
    HandshakeFailure,
    /// When a failed nonce is used by some peer, this message is sent back as evidence.
    LastEdge,
    /// Contains accounts and edge information.
    SyncRoutingTable,
    DistanceVector,
    RequestUpdateNonce,

    SyncAccountsData,

    PeersRequest,
    PeersResponse,

    BlockHeadersRequest,
    BlockHeaders,

    BlockRequest,
    Block,

    Transaction,
    Routed(Box<RoutedMessageV2>),

    /// Gracefully disconnect from other peer.
    Disconnect,
    Challenge,
}

impl From<PeerMessage> for proto::network::PeerMessage {
    fn from(value: PeerMessage) -> Self {
        Self {
            message_type: Some(match value {
                PeerMessage::Tier1Handshake(handshake) => {
                    proto::network::peer_message::Message_type::Tier1Handshake(handshake.into())
                }
                PeerMessage::Tier2Handshake(handshake) => {
                    proto::network::peer_message::Message_type::Tier2Handshake(handshake.into())
                }
                PeerMessage::Routed(routed_message) => {
                    proto::network::peer_message::Message_type::Routed(
                        proto::network::RoutedMessage {
                            borsh: routed_message.msg.try_to_vec().unwrap(),
                            created_at: MessageField::from_option(
                                routed_message.created_at.as_ref().map(utc_to_proto),
                            ),
                            num_hops: None,
                            ..Default::default()
                        },
                    )
                }
                _ => {
                    unreachable!()
                }
            }),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::PeerMessage> for PeerMessage {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: proto::network::PeerMessage) -> Result<Self, Self::Error> {
        Ok(match value.message_type.ok_or("no message type")? {
            Message_type::Tier1Handshake(handshake) =>
                PeerMessage::Tier1Handshake(handshake.try_into().unwrap()),
            Message_type::Tier2Handshake(handshake) =>
                PeerMessage::Tier2Handshake(handshake.try_into().unwrap()),

            Message_type::Routed(message) =>
                PeerMessage::Routed(RoutedMessageV2 {
                    msg: RoutedMessage::try_from_slice(message.borsh.as_slice())?,
                    created_at: message.created_at
                        .as_ref()
                        .map(utc_from_proto)
                        .transpose()?,
                }.into()),
            message_type => { Err(format!("skipped message type {message_type:?}"))? }
        })
    }
}

pub fn utc_to_proto(x: &Utc) -> Timestamp {
    Timestamp {
        seconds: x.unix_timestamp(),
        nanos: x.nanosecond() as i32,
        ..Default::default()
    }
}

pub fn utc_from_proto(x: &Timestamp) -> Result<Utc, time::error::ComponentRange> {
    Utc::from_unix_timestamp_nanos((x.seconds as i128 * 1_000_000_000) + (x.nanos as i128))
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
    use crate::types::peer_message::PeerMessage;

    #[test]
    fn test_serde() -> Result<()> {
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
        let keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
        let sender_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            keypair.public.to_bytes()
        ))
        );
        let target_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            ed25519_dalek::Keypair::generate(&mut OsRng).public.to_bytes())
        ));
        let secret_key = SecretKey::ED25519(ED25519SecretKey(keypair.to_bytes()));
        let partial_edge_info = PartialEdgeInfo::new(
            &sender_peer_id, &target_peer_id, 1, &secret_key,
        );
        let handshake = Handshake {
            protocol_version: 63,
            oldest_supported_version: 61,
            sender_peer_id,
            target_peer_id,
            sender_listen_port: Some(51200),
            sender_chain_info,
            partial_edge_info,
        };

        let peer_message = PeerMessage::Tier2Handshake(handshake);

        let peer_message_original = peer_message.clone();
        let network_peer_message: network::PeerMessage = peer_message.into();
        let peer_message_restored: PeerMessage = network_peer_message.try_into().unwrap();
        assert_eq!(peer_message_original, peer_message_restored);

        Ok(())
    }
}
