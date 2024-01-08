use std::str::FromStr;

use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};
use near_network_primitives::types::{PeerChainInfoV2, PeerInfo};
use near_primitives::block::GenesisId;
use near_primitives::hash::CryptoHash;

#[derive(Debug, Clone, Copy)]
pub enum Network {
    Localnet,
    Testnet,
    Mainnet,
}

impl ValueEnum for Network {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Localnet, Self::Testnet, Self::Mainnet]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Network::Localnet => PossibleValue::new("localnet").help("Localnet"),
            Network::Testnet => PossibleValue::new("testnet").help("Testnet"),
            Network::Mainnet => PossibleValue::new("mainnet").help("Mainnet"),
        }
        .into()
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::value_variants()
            .iter()
            .find_map(|network| {
                network
                    .to_possible_value()?
                    .matches(s, false)
                    .then_some(*network)
            })
            .ok_or("invalid variant".to_owned())
    }
}

impl From<Network> for GenesisId {
    fn from(value: Network) -> Self {
        match value {
            Network::Mainnet => Self {
                chain_id: "mainnet".to_string(),
                hash: CryptoHash([
                    198, 253, 249, 28, 142, 130, 248, 249, 23, 204, 25, 117, 233, 222, 28, 100,
                    190, 17, 137, 158, 50, 29, 253, 245, 254, 188, 251, 183, 49, 63, 20, 134,
                ]),
            },
            Network::Testnet => Self {
                chain_id: "testnet".to_string(),
                hash: CryptoHash([
                    215, 132, 218, 90, 158, 94, 102, 102, 133, 22, 193, 154, 128, 149, 68, 143,
                    197, 74, 34, 162, 137, 113, 220, 51, 15, 0, 153, 223, 148, 55, 148, 16,
                ]),
            },
            Network::Localnet => Self {
                chain_id: "localnet".to_string(),
                hash: CryptoHash([
                    237, 73, 113, 245, 4, 71, 207, 106, 231, 60, 163, 164, 199, 44, 146, 231, 164,
                    52, 93, 229, 128, 78, 65, 244, 67, 198, 245, 244, 136, 235, 2, 203,
                ]),
            },
        }
    }
}

impl From<Network> for PeerChainInfoV2 {
    fn from(value: Network) -> Self {
        Self {
            genesis_id: value.into(),
            height: 0,
            tracked_shards: vec![],
            archival: false,
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(long)]
    pub target_peer_info: PeerInfo,
    #[arg(long)]
    pub network: Network,
    #[arg(long, default_value_t = 63)]
    pub protocol_version: u32,
    #[arg(long, default_value_t = 61)]
    pub oldest_supported_version: u32,
    #[arg(long, default_value_t = 34567)]
    pub sender_listen_port: u16,
}
