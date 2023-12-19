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

#[cfg(test)]
mod tests {
    use near_crypto::{ED25519PublicKey, PublicKey};
    use rand::rngs::OsRng;

    use crate::proto::network;

    #[test]
    fn test_serde() -> std::io::Result<()> {
        let pk = PublicKey::ED25519(ED25519PublicKey::from(
            ed25519_dalek::SecretKey::generate(&mut OsRng).to_bytes())
        );
        let pk_original = pk.clone();
        let proto_pk: network::PublicKey = pk.into();
        let pk_restored: PublicKey = proto_pk.try_into()?;
        assert_eq!(pk_original, pk_restored);
        Ok(())
    }
}