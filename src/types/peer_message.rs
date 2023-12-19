use crate::proto;
use crate::types::handshake::{utc_to_proto, Handshake};
use near_network_primitives::types::RoutedMessageV2;
use near_primitives::borsh::BorshSerialize;
use protobuf::MessageField;

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
        proto::network::PeerMessage {
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
                            //TODO map by lib
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
