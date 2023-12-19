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

#[cfg(test)]
mod tests {
    use near_crypto::{ED25519PublicKey, PublicKey};
    use near_primitives::network::PeerId;
    use rand::rngs::OsRng;

    use crate::proto::network;

    #[test]
    fn test_serde() -> std::io::Result<()> {
        let peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey::from(
            ed25519_dalek::SecretKey::generate(&mut OsRng).to_bytes())
        ));
        let peer_id_original = peer_id.clone();
        let network_peer_id: network::PublicKey = peer_id.into();
        let peer_id_restored: PeerId = network_peer_id.try_into()?;
        assert_eq!(peer_id_original, peer_id_restored);
        Ok(())
    }
}
