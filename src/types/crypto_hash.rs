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
