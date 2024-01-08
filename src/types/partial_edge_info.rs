use near_network_primitives::types::PartialEdgeInfo;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

use crate::proto;

impl From<PartialEdgeInfo> for proto::network::PartialEdgeInfo {
    fn from(value: PartialEdgeInfo) -> Self {
        Self {
            borsh: value.try_to_vec().unwrap(),
            ..Default::default()
        }
    }
}

impl TryFrom<proto::network::PartialEdgeInfo> for PartialEdgeInfo {
    type Error = std::io::Error;

    fn try_from(value: proto::network::PartialEdgeInfo) -> Result<Self, Self::Error> {
        Self::try_from_slice(&value.borsh)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use near_crypto::{ED25519PublicKey, ED25519SecretKey, PublicKey, SecretKey};
    use near_network_primitives::types::PartialEdgeInfo;
    use near_primitives::network::PeerId;
    use rand::rngs::OsRng;

    use crate::proto::network;

    #[test]
    fn test_serde() -> Result<()> {
        let keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
        let sender_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            keypair.public.to_bytes(),
        )));
        let target_peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey(
            ed25519_dalek::Keypair::generate(&mut OsRng)
                .public
                .to_bytes(),
        )));
        let secret_key = SecretKey::ED25519(ED25519SecretKey(keypair.to_bytes()));
        let partial_edge_info =
            PartialEdgeInfo::new(&sender_peer_id, &target_peer_id, 1, &secret_key);
        let partial_edge_info_original = partial_edge_info.clone();
        let network_partial_edge_info: network::PartialEdgeInfo = partial_edge_info.into();
        let partial_edge_info_restored: PartialEdgeInfo =
            network_partial_edge_info.try_into().unwrap();
        assert_eq!(partial_edge_info_original, partial_edge_info_restored);

        Ok(())
    }
}
