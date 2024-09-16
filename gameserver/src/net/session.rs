use anyhow::Result;
use prost::Message;
use proto::CmdID;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use super::{packet::CommandHandler, NetPacket};

pub struct PlayerSession {
    pub(crate) client_socket: TcpStream,
}

impl PlayerSession {
    pub const fn new(client_socket: TcpStream) -> Self {
        Self { client_socket }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let net_packet = NetPacket::read(&mut self.client_socket).await?;
            Self::on_message(self, net_packet.cmd_type, net_packet.body).await?;
        }
    }

    pub async fn send(&mut self, body: impl Message + CmdID) -> Result<()> {
        let mut buf = Vec::new();
        body.encode(&mut buf)?;

        let payload: Vec<u8> = NetPacket {
            cmd_type: body.get_cmd_id(),
            head: Vec::new(),
            body: buf,
        }
        .into();

        self.client_socket.write_all(&payload).await?;
        Ok(())
    }
}

// Auto implemented
impl CommandHandler for PlayerSession {}
