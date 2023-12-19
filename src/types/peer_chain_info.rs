use near_network_primitives::types::PeerChainInfoV2;
use near_primitives::block::GenesisId;
use protobuf::MessageField;
use crate::proto;

impl From<PeerChainInfoV2> for proto::network::PeerChainInfo {
    fn from(value: PeerChainInfoV2) -> Self {
        Self {
            genesis_id: MessageField::some(value.genesis_id.into()),
            height: value.height,
            tracked_shards: value.tracked_shards,
            archival: value.archival,
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::PeerChainInfo> for PeerChainInfoV2 {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: proto::network::PeerChainInfo) -> Result<Self, Self::Error> {
        let genesis_id = GenesisId::default();
        Ok(Self {
            genesis_id,
            height: value.height,
            tracked_shards: value.tracked_shards,
            archival: value.archival,
        })
    }
}
