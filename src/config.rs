use clap::Parser;
use near_network_primitives::types::PeerInfo;

pub const DEFAULT_PORT: u16 = 34567;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(long)]
    pub target_peer_info: PeerInfo,
    #[arg(default_value_t = 63)]
    pub protocol_version: u32,
    #[arg(default_value_t = 61)]
    pub oldest_supported_version: u32,
    #[arg(default_value_t = DEFAULT_PORT)]
    pub sender_listen_port: u16,
}