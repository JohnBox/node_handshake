#![allow(async_fn_in_trait)]
use std::error::Error;
use std::pin::Pin;

use bytes::BytesMut;
use protobuf::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::types::peer_message::PeerMessage;

pub mod config;
mod proto;
pub mod types;

pub trait SendPeerMessage: AsyncWriteExt {
    async fn send_peer_message(
        mut self: Pin<&mut Self>,
        peer_message: PeerMessage,
    ) -> Result<(), Box<dyn Error>> {
        let network_peer_message: proto::network::PeerMessage = peer_message.into();

        let message = network_peer_message.write_to_bytes()?;
        let message_size = message.len();

        self.write_u32_le(message_size as u32).await?;
        self.write_all(&message).await?;
        self.flush().await?;

        Ok(())
    }
}

pub trait ReceivePeerMessage: AsyncReadExt {
    async fn receive_peer_message(mut self: Pin<&mut Self>) -> Result<PeerMessage, Box<dyn Error>> {
        let message_size = self.read_u32_le().await? as usize;

        let mut buf = BytesMut::with_capacity(message_size);
        self.read_buf(&mut buf).await?;

        let message = proto::network::PeerMessage::parse_from_bytes(&buf)?;
        message.try_into()
    }
}

impl SendPeerMessage for TcpStream {}

impl ReceivePeerMessage for TcpStream {}
