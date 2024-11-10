use std::{
    io::Error,
    net::SocketAddr,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
    time::Duration,
};

use anyhow::Result;
use mhy_kcp::Kcp;
use prost::Message;
use proto::{ClientDownloadData, ClientDownloadDataScNotify, CmdID, CmdPlayerType};
use tokio::{io::AsyncWrite, net::UdpSocket, sync::Mutex};

use crate::util;

use super::{packet::CommandHandler, NetPacket};

struct RemoteEndPoint {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
}

#[derive(Clone)]
pub struct PlayerSession {
    pub token: u32,
    kcp: Arc<Mutex<Kcp<RemoteEndPoint>>>,
    start_time: u64,
    pub dvd_running: Arc<AtomicBool>,
    pub is_destroyed: bool,
}

impl PlayerSession {
    pub fn new(socket: Arc<UdpSocket>, addr: SocketAddr, conv: u32, token: u32) -> Self {
        Self {
            token,
            kcp: Arc::new(Mutex::new(Kcp::new(
                conv,
                token,
                false,
                RemoteEndPoint { socket, addr },
            ))),
            start_time: util::cur_timestamp_secs(),
            dvd_running: Arc::new(AtomicBool::new(false)),
            is_destroyed: false,
        }
    }

    pub async fn toggle_dvd(&mut self) {
        if self.dvd_running.swap(true, Ordering::SeqCst) {
            self.dvd_running.store(false, Ordering::SeqCst);
            self.send(ClientDownloadDataScNotify {
                download_data: Some(ClientDownloadData {
                    version: 51,
                    time: util::cur_timestamp_ms() as i64,
                    data: rbase64::decode("bG9jYWwgZnVuY3Rpb24gZGVzdHJveSgpCiAgICBsb2NhbCBleGlzdGluZ0NhbnZhc0dPID0gQ1MuVW5pdHlFbmdpbmUuR2FtZU9iamVjdC5GaW5kKCJJbWFnZUJveENhbnZhcyIpCiAgICBpZiBleGlzdGluZ0NhbnZhc0dPIHRoZW4KICAgICAgICBDUy5Vbml0eUVuZ2luZS5PYmplY3QuRGVzdHJveShleGlzdGluZ0NhbnZhc0dPKQogICAgZW5kCgogICAgQ1NIQVJQX0xJU1RfQ0xBU1MgPSBuaWwKICAgIFRFWFRVUkVfTElTVCA9IG5pbAogICAgQ09NUE9ORU5UX0xJU1QgPSBuaWwKICAgIERJUkVDVElPTiA9IG5pbAogICAgU1BFRUQgPSBuaWwKZW5kCgp4cGNhbGwoZGVzdHJveSwgZnVuY3Rpb24oZXJyb3IpCiAgICBDUy5NaUhvWW8uU0RLLk5ldHdvcmtNYW5hZ2VyLlNob3dOZXR3b3JrRXJyb3IoMCwgdG9zdHJpbmcoZXJyb3IpKQplbmQpCg==").unwrap(),
                    njddjhapnbo: 0
                })
            }).await.unwrap();
            return;
        }

        let dvd_running = Arc::clone(&self.dvd_running);
        let session = self.clone();

        tokio::spawn(async move {
            let body = ClientDownloadDataScNotify {
                download_data: Some(ClientDownloadData {
                    version: 51,
                    time: util::cur_timestamp_ms() as i64,
                    data: rbase64::decode("bG9jYWwgZnVuY3Rpb24gbWFpbigpCiAgICBpZiBub3QgVEVYVFVSRV9MSVNUIG9yIG5vdCBDT01QT05FTlRfTElTVCB0aGVuCiAgICAgICAgLS0gY2hlY2sgaWYgaGF2ZSBpbWFnZQogICAgICAgIGlmIG5vdCBDUy5TeXN0ZW0uSU8uRmlsZS5FeGlzdHMoImltYWdlLnBuZyIpIHRoZW4KICAgICAgICAgICAgQ1MuTWlIb1lvLlNESy5OZXR3b3JrTWFuYWdlci5TaG93TmV0d29ya0Vycm9yKDAsCiAgICAgICAgICAgICAgICAiaW1hZ2UucG5nIG5vdCBleGlzdCEgcGxlYXNlIHB1dCBpbWFnZS5wbmcgaW4geW91ciBnYW1lIGRpcmVjdG9yeSEiKQogICAgICAgICAgICByZXR1cm4KICAgICAgICBlbmQKCiAgICAgICAgQ1MuVW5pdHlFbmdpbmUuQXBwbGljYXRpb24udGFyZ2V0RnJhbWVSYXRlID0gMzYwCiAgICAgICAgQ1MuVW5pdHlFbmdpbmUuUXVhbGl0eVNldHRpbmdzLnZTeW5jQ291bnQgPSAwCgogICAgICAgIENTSEFSUF9MSVNUX0NMQVNTID0gQ1MuU3lzdGVtLkNvbGxlY3Rpb25zLkdlbmVyaWMuTGlzdChDUy5TeXN0ZW0uT2JqZWN0KQogICAgICAgIFRFWFRVUkVfTElTVCA9IENTSEFSUF9MSVNUX0NMQVNTKCkKICAgICAgICBDT01QT05FTlRfTElTVCA9IENTSEFSUF9MSVNUX0NMQVNTKCkKICAgICAgICBESVJFQ1RJT04gPSAic2UiCiAgICAgICAgU1BFRUQgPSA1CgogICAgICAgIGxvY2FsIGltYWdlQnl0ZXMgPSBDUy5TeXN0ZW0uSU8uRmlsZS5SZWFkQWxsQnl0ZXMoImltYWdlLnBuZyIpCiAgICAgICAgbG9jYWwgdGV4dHVyZSA9IENTLlVuaXR5RW5naW5lLlRleHR1cmUyRCgyLCAyKQogICAgICAgIENTLlVuaXR5RW5naW5lLkltYWdlQ29udmVyc2lvbi5Mb2FkSW1hZ2UodGV4dHVyZSwgaW1hZ2VCeXRlcykKICAgICAgICBURVhUVVJFX0xJU1Q6QWRkKHRleHR1cmUpCgogICAgICAgIGxvY2FsIGV4aXN0aW5nQ2FudmFzR08gPSBDUy5Vbml0eUVuZ2luZS5HYW1lT2JqZWN0LkZpbmQoIkltYWdlQm94Q2FudmFzIikKICAgICAgICBpZiBleGlzdGluZ0NhbnZhc0dPIHRoZW4KICAgICAgICAgICAgQ1MuVW5pdHlFbmdpbmUuT2JqZWN0LkRlc3Ryb3koZXhpc3RpbmdDYW52YXNHTykKICAgICAgICBlbmQKCiAgICAgICAgbG9jYWwgY2FudmFzR08gPSBDUy5Vbml0eUVuZ2luZS5HYW1lT2JqZWN0KCJJbWFnZUJveENhbnZhcyIpCiAgICAgICAgbG9jYWwgY2FudmFzID0gY2FudmFzR086QWRkQ29tcG9uZW50KHR5cGVvZihDUy5Vbml0eUVuZ2luZS5DYW52YXMpKQogICAgICAgIGNhbnZhcy5yZW5kZXJNb2RlID0gQ1MuVW5pdHlFbmdpbmUuUmVuZGVyTW9kZS5TY3JlZW5TcGFjZU92ZXJsYXkKCgogICAgICAgIGxvY2FsIGNhbnZhc1NjYWxlciA9IGNhbnZhc0dPOkFkZENvbXBvbmVudCh0eXBlb2YoQ1MuVW5pdHlFbmdpbmUuVUkuQ2FudmFzU2NhbGVyKSkKICAgICAgICBjYW52YXNTY2FsZXIudWlTY2FsZU1vZGUgPSBDUy5Vbml0eUVuZ2luZS5VSS5DYW52YXNTY2FsZXIuU2NhbGVNb2RlLlNjYWxlV2l0aFNjcmVlblNpemUKICAgICAgICBjYW52YXNTY2FsZXIucmVmZXJlbmNlUmVzb2x1dGlvbiA9IENTLlVuaXR5RW5naW5lLlZlY3RvcjIoQ1MuVW5pdHlFbmdpbmUuU2NyZWVuLndpZHRoLAogICAgICAgICAgICBDUy5Vbml0eUVuZ2luZS5TY3JlZW4uaGVpZ2h0KQoKCiAgICAgICAgbG9jYWwgaW1hZ2VCb3hHTyA9IENTLlVuaXR5RW5naW5lLkdhbWVPYmplY3QoIkltYWdlQm94IikKICAgICAgICBpbWFnZUJveEdPLnRyYW5zZm9ybS5wYXJlbnQgPSBjYW52YXNHTy50cmFuc2Zvcm0KCiAgICAgICAgbG9jYWwgaW1hZ2UgPSBpbWFnZUJveEdPOkFkZENvbXBvbmVudCh0eXBlb2YoQ1MuVW5pdHlFbmdpbmUuVUkuSW1hZ2UpKQoKICAgICAgICBsb2NhbCB0ZXh0dXJlID0gVEVYVFVSRV9MSVNUWzBdCgogICAgICAgIGltYWdlLnNwcml0ZSA9IENTLlVuaXR5RW5naW5lLlNwcml0ZS5DcmVhdGUodGV4dHVyZSwgQ1MuVW5pdHlFbmdpbmUuUmVjdCgwLCAwLCB0ZXh0dXJlLndpZHRoLCB0ZXh0dXJlLmhlaWdodCksCiAgICAgICAgICAgIENTLlVuaXR5RW5naW5lLlZlY3RvcjIoMC41LCAwLjUpKQoKICAgICAgICBDT01QT05FTlRfTElTVDpBZGQoaW1hZ2VCb3hHTzpHZXRDb21wb25lbnQodHlwZW9mKENTLlVuaXR5RW5naW5lLlJlY3RUcmFuc2Zvcm0pKSkKICAgICAgICBDT01QT05FTlRfTElTVFswXS5zaXplRGVsdGEgPSBDUy5Vbml0eUVuZ2luZS5WZWN0b3IyKHRleHR1cmUud2lkdGggLyAyLCB0ZXh0dXJlLmhlaWdodCAvIDIpCiAgICAgICAgQ09NUE9ORU5UX0xJU1RbMF0uYW5jaG9yZWRQb3NpdGlvbiA9IENTLlVuaXR5RW5naW5lLlZlY3RvcjIoMCwgMCkKICAgIGVsc2UKICAgICAgICBsb2NhbCBpbWFnZVJlY3RUcmFuc2Zvcm0gPSBDT01QT05FTlRfTElTVFswXQogICAgICAgIGxvY2FsIG1fcmlnaHQgPSAoQ1MuVW5pdHlFbmdpbmUuU2NyZWVuLndpZHRoIC8gMikgLSBpbWFnZVJlY3RUcmFuc2Zvcm0uc2l6ZURlbHRhLnggLyAyIC0tIE1heCByaWdodCBib3VuZGFyeQogICAgICAgIGxvY2FsIG1fdG9wID0gKENTLlVuaXR5RW5naW5lLlNjcmVlbi5oZWlnaHQgLyAyKSAtIGltYWdlUmVjdFRyYW5zZm9ybS5zaXplRGVsdGEueSAvIDIgIC0tIE1heCB0b3AgYm91bmRhcnkKICAgICAgICBsb2NhbCBtX2JvdCA9IC1tX3RvcCAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAtLSBCb3R0b20gYm91bmRhcnkKICAgICAgICBsb2NhbCBtX2xlZnQgPSAtbV9yaWdodCAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAtLSBMZWZ0IGJvdW5kYXJ5CiAgICAgICAgbG9jYWwgcG9zaXRpb24gPSBpbWFnZVJlY3RUcmFuc2Zvcm0uYW5jaG9yZWRQb3NpdGlvbgogICAgICAgIGxvY2FsIHggPSBwb3NpdGlvbi54CiAgICAgICAgbG9jYWwgeSA9IHBvc2l0aW9uLnkKCiAgICAgICAgLS0gTW92ZSBiYXNlZCBvbiBkaXJlY3Rpb24KICAgICAgICBpZiBESVJFQ1RJT04gPT0gIm5lIiB0aGVuCiAgICAgICAgICAgIHggPSB4ICsgU1BFRUQKICAgICAgICAgICAgeSA9IHkgLSBTUEVFRAogICAgICAgIGVsc2VpZiBESVJFQ1RJT04gPT0gIm53IiB0aGVuCiAgICAgICAgICAgIHggPSB4IC0gU1BFRUQKICAgICAgICAgICAgeSA9IHkgLSBTUEVFRAogICAgICAgIGVsc2VpZiBESVJFQ1RJT04gPT0gInNlIiB0aGVuCiAgICAgICAgICAgIHggPSB4ICsgU1BFRUQKICAgICAgICAgICAgeSA9IHkgKyBTUEVFRAogICAgICAgIGVsc2VpZiBESVJFQ1RJT04gPT0gInN3IiB0aGVuCiAgICAgICAgICAgIHggPSB4IC0gU1BFRUQKICAgICAgICAgICAgeSA9IHkgKyBTUEVFRAogICAgICAgIGVuZAoKICAgICAgICAtLSBCb3R0b20gYm91bmRhcnkgY2hlY2sgKHdoZW4gbW92aW5nIGRvd253YXJkKQogICAgICAgIGlmIHkgPD0gbV9ib3QgdGhlbgogICAgICAgICAgICBpZiBESVJFQ1RJT04gPT0gIm53IiB0aGVuCiAgICAgICAgICAgICAgICBESVJFQ1RJT04gPSAic3ciCiAgICAgICAgICAgIGVsc2VpZiBESVJFQ1RJT04gPT0gIm5lIiB0aGVuCiAgICAgICAgICAgICAgICBESVJFQ1RJT04gPSAic2UiCiAgICAgICAgICAgIGVuZAogICAgICAgICAgICB5ID0gbV9ib3QgLS0gRml4IHRoZSBwb3NpdGlvbiB0byBiZSBleGFjdGx5IGF0IHRoZSBib3R0b20gYm91bmRhcnkKICAgICAgICBlbmQKCiAgICAgICAgLS0gVG9wIGJvdW5kYXJ5IGNoZWNrICh3aGVuIG1vdmluZyB1cHdhcmQpCiAgICAgICAgaWYgeSA+PSBtX3RvcCB0aGVuCiAgICAgICAgICAgIGlmIERJUkVDVElPTiA9PSAic2UiIHRoZW4KICAgICAgICAgICAgICAgIERJUkVDVElPTiA9ICJuZSIKICAgICAgICAgICAgZWxzZWlmIERJUkVDVElPTiA9PSAic3ciIHRoZW4KICAgICAgICAgICAgICAgIERJUkVDVElPTiA9ICJudyIKICAgICAgICAgICAgZW5kCiAgICAgICAgICAgIHkgPSBtX3RvcCAtLSBGaXggdGhlIHBvc2l0aW9uIHRvIGJlIGV4YWN0bHkgYXQgdGhlIHRvcCBib3VuZGFyeQogICAgICAgIGVuZAoKICAgICAgICAtLSBMZWZ0IGJvdW5kYXJ5IGNoZWNrICh3aGVuIG1vdmluZyBsZWZ0KQogICAgICAgIGlmIHggPD0gbV9sZWZ0IHRoZW4KICAgICAgICAgICAgaWYgRElSRUNUSU9OID09ICJudyIgdGhlbgogICAgICAgICAgICAgICAgRElSRUNUSU9OID0gIm5lIgogICAgICAgICAgICBlbHNlaWYgRElSRUNUSU9OID09ICJzdyIgdGhlbgogICAgICAgICAgICAgICAgRElSRUNUSU9OID0gInNlIgogICAgICAgICAgICBlbmQKICAgICAgICAgICAgeCA9IG1fbGVmdCAtLSBGaXggdGhlIHBvc2l0aW9uIHRvIGJlIGV4YWN0bHkgYXQgdGhlIGxlZnQgYm91bmRhcnkKICAgICAgICBlbmQKCiAgICAgICAgLS0gUmlnaHQgYm91bmRhcnkgY2hlY2sgKHdoZW4gbW92aW5nIHJpZ2h0KQogICAgICAgIGlmIHggPj0gbV9yaWdodCB0aGVuCiAgICAgICAgICAgIGlmIERJUkVDVElPTiA9PSAibmUiIHRoZW4KICAgICAgICAgICAgICAgIERJUkVDVElPTiA9ICJudyIKICAgICAgICAgICAgZWxzZWlmIERJUkVDVElPTiA9PSAic2UiIHRoZW4KICAgICAgICAgICAgICAgIERJUkVDVElPTiA9ICJzdyIKICAgICAgICAgICAgZW5kCiAgICAgICAgICAgIHggPSBtX3JpZ2h0IC0tIEZpeCB0aGUgcG9zaXRpb24gdG8gYmUgZXhhY3RseSBhdCB0aGUgcmlnaHQgYm91bmRhcnkKICAgICAgICBlbmQKCiAgICAgICAgLS0gVXBkYXRlIHBvc2l0aW9uCiAgICAgICAgaW1hZ2VSZWN0VHJhbnNmb3JtLmFuY2hvcmVkUG9zaXRpb24gPSBDUy5Vbml0eUVuZ2luZS5WZWN0b3IyKHgsIHkpCiAgICBlbmQKZW5kCgp4cGNhbGwobWFpbiwgZnVuY3Rpb24oZXJyb3IpCiAgICBDUy5NaUhvWW8uU0RLLk5ldHdvcmtNYW5hZ2VyLlNob3dOZXR3b3JrRXJyb3IoMCwgdG9zdHJpbmcoZXJyb3IpKQplbmQpCg==").unwrap(),
                    njddjhapnbo: 0
                })
            };
            while dvd_running.load(Ordering::SeqCst) {
                session.send(body.clone()).await.unwrap();
                std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0)); // 60fps
            }
        });
    }

    pub async fn consume(&mut self, buffer: &[u8]) -> Result<()> {
        {
            let mut kcp = self.kcp.lock().await;
            kcp.input(buffer)?;
            kcp.async_update(self.session_time() as u32).await?;
            kcp.async_flush().await?;
        }

        let mut packets = Vec::new();
        let mut buf = [0; 24756];
        while let Ok(length) = self.kcp.lock().await.recv(&mut buf) {
            packets.push(NetPacket::from(&buf[..length]));
        }

        for packet in packets {
            // TODO: Temporary fix
            if packet.cmd_type == CmdPlayerType::CmdPlayerLogoutCsReq as u16 {
                self.is_destroyed = true;
                self.dvd_running.store(false, Ordering::SeqCst);
                return Ok(());
            };
            Self::on_message(self, packet.cmd_type, packet.body).await?;
        }

        self.kcp
            .lock()
            .await
            .async_update(self.session_time() as u32)
            .await?;
        Ok(())
    }

    pub async fn send(&self, body: impl Message + CmdID) -> Result<()> {
        let mut buf = Vec::new();
        body.encode(&mut buf)?;

        let payload: Vec<u8> = NetPacket {
            cmd_type: body.get_cmd_id(),
            head: Vec::new(),
            body: buf,
        }
        .into();

        let mut kcp = self.kcp.lock().await;
        kcp.send(&payload)?;
        kcp.async_flush().await?;
        kcp.async_update(self.session_time() as u32).await?;

        Ok(())
    }

    pub async fn send_raw(&self, payload: NetPacket) -> Result<()> {
        let mut kcp = self.kcp.lock().await;
        let payload: Vec<u8> = payload.into();
        kcp.send(&payload)?;
        kcp.async_flush().await?;
        kcp.async_update(self.session_time() as u32).await?;

        Ok(())
    }

    fn session_time(&self) -> u64 {
        util::cur_timestamp_secs() - self.start_time
    }
}

// Auto implemented
impl CommandHandler for PlayerSession {}

impl AsyncWrite for RemoteEndPoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.socket.poll_send_to(cx, buf, self.addr)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}
