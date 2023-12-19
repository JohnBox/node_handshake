use near_primitives::block::GenesisId;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::hash::CryptoHash;
use protobuf::MessageField;

use crate::proto;

impl From<GenesisId> for proto::network::GenesisId {
    fn from(value: GenesisId) -> Self {
        Self {
            chain_id: value.chain_id,
            hash: MessageField::some(value.hash.into()),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::GenesisId> for GenesisId {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: proto::network::GenesisId) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            hash: CryptoHash::try_from_slice(value.hash.hash.as_slice())?,
        })
    }
}
