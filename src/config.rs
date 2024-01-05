use std::str::FromStr;
use clap::{Parser, ValueEnum};
use clap::builder::PossibleValue;
use near_network_primitives::types::{PeerChainInfoV2, PeerInfo};
use near_primitives::block::GenesisId;
use near_primitives::hash::CryptoHash;


#[derive(Debug, Clone, Copy, Default)]
pub enum Network {
    #[default]
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
        }.into()
    }
}

impl FromStr for Network {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::value_variants()
            .iter()
            .find_map(|network| network.to_possible_value()?.matches(s, false).then_some(*network))
            .ok_or("invalid variant".to_owned())
    }
}

impl From<Network> for GenesisId {
    fn from(value: Network) -> Self {
        match value {
            Network::Mainnet => Self {
                chain_id: "mainnet".to_string(),
                // TODO change to mainnet hash
                hash: CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap(),
            },
            Network::Testnet => Self {
                chain_id: "testnet".to_string(),
                // TODO change to testnet hash
                hash: CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap(),
            },
            Network::Localnet => Self {
                chain_id: "localnet".to_string(),
                hash: CryptoHash::from_str("GyGacsMkHfq1n1HQ3mHF4xXqAMTDR183FnckCaZ2r5yL").unwrap(),
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
    #[arg(default_value_t = 63)]
    pub protocol_version: u32,
    #[arg(default_value_t = 61)]
    pub oldest_supported_version: u32,
    #[arg(default_value_t = 34567)]
    pub sender_listen_port: u16,
}