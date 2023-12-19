use near_crypto::PublicKey;
use near_primitives::network::PeerId;

use crate::proto;

impl From<PeerId> for proto::network::PublicKey {
    fn from(value: PeerId) -> Self {
        value.public_key().clone().into()
    }
}

impl TryFrom<proto::network::PublicKey> for PeerId {
    type Error = std::io::Error;
    fn try_from(value: proto::network::PublicKey) -> Result<Self, Self::Error> {
        PublicKey::try_from(value).map(Self::new)
    }
}
