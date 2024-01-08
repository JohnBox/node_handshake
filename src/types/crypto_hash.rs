use near_primitives::hash::CryptoHash;

use crate::proto;

impl From<CryptoHash> for proto::network::CryptoHash {
    fn from(value: CryptoHash) -> Self {
        Self {
            hash: value.0.into(),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::CryptoHash> for CryptoHash {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: proto::network::CryptoHash) -> Result<Self, Self::Error> {
        Self::try_from(value.hash.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use near_primitives::borsh::BorshDeserialize;
    use near_primitives::hash::CryptoHash;
    use rand::rngs::OsRng;

    use crate::proto::network;

    use anyhow::Result;

    #[test]
    fn test_serde() -> Result<()> {
        let crypto_hash =
            CryptoHash::try_from_slice(ed25519_dalek::SecretKey::generate(&mut OsRng).as_bytes())?;

        let crypto_hash_original = crypto_hash.clone();
        let network_crypto_hash: network::CryptoHash = crypto_hash.into();
        let crypto_hash_restored: CryptoHash = network_crypto_hash.try_into().unwrap();
        assert_eq!(crypto_hash_original, crypto_hash_restored);
        Ok(())
    }
}
