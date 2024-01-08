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

#[cfg(test)]
mod tests {
    use anyhow::Result;
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
                ed25519_dalek::SecretKey::generate(&mut OsRng).as_bytes(),
            )?,
        };
        let genesis_id_original = genesis_id.clone();
        let network_genesis_id: network::GenesisId = genesis_id.into();
        let genesis_id_restored: GenesisId = network_genesis_id.try_into().unwrap();
        assert_eq!(genesis_id_original, genesis_id_restored);

        Ok(())
    }
}
