use near_network_primitives::types::PeerChainInfoV2;
use near_primitives::block::GenesisId;
use near_primitives::borsh::BorshDeserialize;
use near_primitives::hash::CryptoHash;
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
        let genesis_id = GenesisId {
            chain_id: value.genesis_id.chain_id.clone(),
            hash: CryptoHash::try_from_slice(
                value.genesis_id.hash.hash.as_slice()
            )?,
        };
        Ok(Self {
            genesis_id,
            height: value.height,
            tracked_shards: value.tracked_shards,
            archival: value.archival,
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use near_network_primitives::types::PeerChainInfoV2;
    use near_primitives::block::GenesisId;
    use near_primitives::borsh::BorshDeserialize;
    use near_primitives::hash::CryptoHash;
    use rand::rngs::OsRng;

    use crate::proto::network;

    #[test]
    fn test_serde() -> Result<()> {
        let genesis_id = GenesisId {
            chain_id: "testnet".to_string(),
            hash: CryptoHash::try_from_slice(
                ed25519_dalek::SecretKey::generate(&mut OsRng).as_bytes()
            )?,
        };
        let peer_chain_info = PeerChainInfoV2 {
            genesis_id,
            height: 0,
            tracked_shards: vec![],
            archival: false,
        };
        let peer_chain_info_original = peer_chain_info.clone();
        let network_peer_chain_info: network::PeerChainInfo = peer_chain_info.into();
        let peer_chain_info_restored: PeerChainInfoV2 = network_peer_chain_info.try_into().unwrap();
        assert_eq!(peer_chain_info_original, peer_chain_info_restored);

        Ok(())
    }
}
