use near_network_primitives::types::PartialEdgeInfo;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

use crate::proto;

impl From<PartialEdgeInfo> for proto::network::PartialEdgeInfo {
    fn from(value: PartialEdgeInfo) -> Self {
        Self {
            borsh: value.try_to_vec().unwrap(),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::PartialEdgeInfo> for PartialEdgeInfo {
    type Error = std::io::Error;

    fn try_from(value: proto::network::PartialEdgeInfo) -> Result<Self, Self::Error> {
        Self::try_from_slice(&value.borsh)
    }
}
