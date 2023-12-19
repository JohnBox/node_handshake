use near_crypto::PublicKey;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

use crate::proto;

impl From<PublicKey> for proto::network::PublicKey {
    fn from(value: PublicKey) -> Self {
        Self {
            borsh: value.try_to_vec().unwrap(),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::PublicKey> for PublicKey {
    type Error = std::io::Error;

    fn try_from(value: proto::network::PublicKey) -> Result<Self, Self::Error> {
        Self::try_from_slice(value.borsh.as_slice())
    }
}
