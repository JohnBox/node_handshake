use near_network_primitives::time;
use near_network_primitives::time::Utc;
use near_network_primitives::types::{
    Edge, PartialEdgeInfo, PeerInfo, RoutedMessage, RoutedMessageV2,
};
use near_primitives::block::{Block, GenesisId};
use near_primitives::block_header::BlockHeader;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};
use near_primitives::challenge::Challenge;
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::SignedTransaction;
use protobuf::well_known_types::timestamp::Timestamp;
use protobuf::MessageField;

use crate::proto;
use crate::proto::network::peer_message::Message_type;
use crate::proto::network::{
    Disconnect, DistanceVector, PeersRequest, PeersResponse, RoutingTableUpdate, SyncAccountsData,
};
use crate::types::handshake::Handshake;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum HandshakeFailureReason {
    ProtocolVersionMismatch {
        version: u32,
        oldest_supported_version: u32,
    },
    GenesisMismatch(GenesisId),
    InvalidTarget,
}

#[derive(PartialEq, Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum PeerMessage {
    Tier1Handshake(Handshake),
    Tier2Handshake(Handshake),
    HandshakeFailure(PeerInfo, HandshakeFailureReason),
    /// When a failed nonce is used by some peer, this message is sent back as evidence.
    LastEdge(Edge),
    /// Contains accounts and edge information.
    SyncRoutingTable(RoutingTableUpdate),
    DistanceVector(DistanceVector),
    RequestUpdateNonce(PartialEdgeInfo),

    SyncAccountsData(SyncAccountsData),

    PeersRequest(PeersRequest),
    PeersResponse(PeersResponse),

    BlockHeadersRequest(Vec<CryptoHash>),
    BlockHeaders(Vec<BlockHeader>),

    BlockRequest(CryptoHash),
    Block(Block),

    Transaction(SignedTransaction),
    Routed(Box<RoutedMessageV2>),

    /// Gracefully disconnect from other peer.
    Disconnect(Disconnect),
    Challenge(Challenge),
}

impl From<PeerMessage> for proto::network::PeerMessage {
    fn from(value: PeerMessage) -> Self {
        Self {
            message_type: Some(match value {
                PeerMessage::Tier1Handshake(handshake) => {
                    Message_type::Tier1Handshake(handshake.into())
                }
                PeerMessage::Tier2Handshake(handshake) => {
                    Message_type::Tier2Handshake(handshake.into())
                }
                PeerMessage::Routed(routed_message) => {
                    Message_type::Routed(proto::network::RoutedMessage {
                        borsh: routed_message.msg.try_to_vec().unwrap(),
                        created_at: MessageField::from_option(
                            routed_message.created_at.as_ref().map(utc_to_proto),
                        ),
                        num_hops: None,
                        ..Default::default()
                    })
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
            Message_type::Tier1Handshake(handshake) => {
                PeerMessage::Tier1Handshake(handshake.try_into().unwrap())
            }
            Message_type::Tier2Handshake(handshake) => {
                PeerMessage::Tier2Handshake(handshake.try_into().unwrap())
            }

            Message_type::Routed(message) => PeerMessage::Routed(
                RoutedMessageV2 {
                    msg: RoutedMessage::try_from_slice(message.borsh.as_slice())?,
                    created_at: message
                        .created_at
                        .as_ref()
                        .map(utc_from_proto)
                        .transpose()?,
                }
                .into(),
            ),
            Message_type::HandshakeFailure(_) => {
                println!("<<< Receive HandshakeFailure");
                Err("skipped message type")?
            }
            Message_type::LastEdge(_) => {
                println!("<<< Receive LastEdge");
                Err("skipped message type")?
            }
            Message_type::SyncRoutingTable(_) => {
                println!("<<< Receive SyncRoutingTable");
                Err("skipped message type")?
            }
            Message_type::DistanceVector(_) => {
                println!("<<< Receive DistanceVector");
                Err("skipped message type")?
            }
            Message_type::UpdateNonceRequest(_) => {
                println!("<<< Receive UpdateNonceRequest");
                Err("skipped message type")?
            }
            Message_type::UpdateNonceResponse(_) => {
                println!("<<< Receive UpdateNonceResponse");
                Err("skipped message type")?
            }
            Message_type::SyncAccountsData(_) => {
                println!("<<< Receive SyncAccountsData");
                Err("skipped message type")?
            }
            Message_type::PeersRequest(_) => {
                println!("<<< Receive PeersRequest");
                Err("skipped message type")?
            }
            Message_type::PeersResponse(_) => {
                println!("<<< Receive PeersResponse");
                Err("skipped message type")?
            }
            Message_type::BlockHeadersRequest(_) => {
                println!("<<< Receive BlockHeadersRequest");
                Err("skipped message type")?
            }
            Message_type::BlockHeadersResponse(_) => {
                println!("<<< Receive BlockHeadersResponse");
                Err("skipped message type")?
            }
            Message_type::BlockRequest(_) => {
                println!("<<< Receive BlockRequest");
                Err("skipped message type")?
            }
            Message_type::BlockResponse(_) => {
                println!("<<< Receive BlockResponse");
                Err("skipped message type")?
            }
            Message_type::Transaction(_) => {
                println!("<<< Receive Transaction");
                Err("skipped message type")?
            }
            Message_type::Disconnect(_) => {
                println!("<<< Receive Disconnect");
                Err("skipped message type")?
            }
            Message_type::Challenge(_) => {
                println!("<<< Receive Challenge");
                Err("skipped message type")?
            }
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

        let peer_message = PeerMessage::Tier2Handshake(handshake);

        let peer_message_original = peer_message.clone();
        let network_peer_message: network::PeerMessage = peer_message.into();
        let peer_message_restored: PeerMessage = network_peer_message.try_into().unwrap();
        assert_eq!(peer_message_original, peer_message_restored);

        Ok(())
    }
}
