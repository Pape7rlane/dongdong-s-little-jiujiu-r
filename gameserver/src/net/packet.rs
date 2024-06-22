use anyhow::Result;
use paste::paste;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::Instrument;

use proto::*;

use super::handlers::*;
use super::PlayerSession;

const HEAD_MAGIC: u32 = 0x9D74C714;
const TAIL_MAGIC: u32 = 0xD7A152C8;

pub struct NetPacket {
    pub cmd_type: u16,
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

impl From<NetPacket> for Vec<u8> {
    fn from(value: NetPacket) -> Self {
        let mut out = Self::new();

        out.extend(HEAD_MAGIC.to_be_bytes());
        out.extend(value.cmd_type.to_be_bytes());
        out.extend((value.head.len() as u16).to_be_bytes());
        out.extend((value.body.len() as u32).to_be_bytes());
        out.extend(value.head);
        out.extend(value.body);
        out.extend(TAIL_MAGIC.to_be_bytes());
        out
    }
}

impl NetPacket {
    pub async fn read(stream: &mut TcpStream) -> std::io::Result<Self> {
        assert_eq!(stream.read_u32().await?, HEAD_MAGIC);
        let cmd_type = stream.read_u16().await?;

        let head_length = stream.read_u16().await? as usize;
        let body_length = stream.read_u32().await? as usize;

        let mut head = vec![0; head_length];
        stream.read_exact(&mut head).await?;

        let mut body = vec![0; body_length];
        stream.read_exact(&mut body).await?;

        assert_eq!(stream.read_u32().await?, TAIL_MAGIC);

        Ok(Self {
            cmd_type,
            head,
            body,
        })
    }
}

macro_rules! trait_handler {
    ($($name:ident $cmd_type:expr;)*) => {
        pub trait CommandHandler {
            $(
                paste! {
                    async fn [<on_$name:snake>](session: &mut PlayerSession, body: &$name) -> Result<()> {
                        [<on_$name:snake>](session, body).await
                    }
                }
            )*

            async fn on_message(session: &mut PlayerSession, cmd_type: u16, payload: Vec<u8>) -> Result<()> {
                use ::prost::Message;
                if PlayerSession::should_send_dummy_rsp(cmd_type) {
                    session.send_dummy_response(cmd_type).await?;
                    return Ok(());
                }
                match cmd_type {
                    $(
                        $cmd_type => {
                            let body = $name::decode(&mut &payload[..])?;
                            paste! {
                                Self::[<on_$name:snake>](session, &body)
                                    .instrument(tracing::info_span!(stringify!([<on_$name:snake>]), cmd_type = cmd_type))
                                    .await
                            }
                        }
                    )*
                    _ => {
                        tracing::warn!("Unknown command type: {cmd_type}");
                        Ok(())
                    },
                }
            }
        }
    };
}

trait_handler! {
    PlayerGetTokenCsReq 58;         // PlayerGetTokenScRsp
    PlayerLoginCsReq 18;            // PlayerLoginScRsp, PlayerBasicInfo
    GetMissionStatusCsReq 1248;     // GetMissionStatusScRsp, Mission
    GetBasicInfoCsReq 42;           // GetBasicInfoScRsp, PlayerSettingInfo
    // GetHeroBasicTypeInfoCsReq 68;   // GetHeroBasicTypeInfoScRsp, HeroBasicTypeInfo
    GetMultiPathAvatarInfoCsReq 69;
    GetAvatarDataCsReq 318;         // GetAvatarDataScRsp, Avatar
    GetAllLineupDataCsReq 748;      // GetAllLineupDataScRsp, LineupInfo, ExtraLineupType, LineupAvatar, AmountInfo
    GetCurLineupDataCsReq 729;      // GetCurLineupDataScRsp
    GetCurSceneInfoCsReq 1428;      // GetCurSceneInfoScRsp, SceneInfo
    PlayerHeartBeatCsReq 33;        // PlayerHeartBeatScRsp

    // // Tutorial (dummy!)
    // GetTutorialGuideCsReq 1691;
    // UnlockTutorialGuideCsReq 1630;
    // GetTutorialCsReq 1661;

    // Entity move (dummy!)
    SceneEntityMoveCsReq 1418;

    // // Inventory (dummy!)
    GetBagCsReq 518;
    GetArchiveDataCsReq 2318;
    DressAvatarCsReq 319;
    TakeOffEquipmentCsReq 389;
    DressRelicAvatarCsReq 322;
    TakeOffRelicCsReq 362;

    // // Chat (dummy!)
    // SendMsgCsReq 3918;
    GetPrivateChatHistoryCsReq 3958;
    GetFriendListInfoCsReq 2918;

    // // In-game lineup
    JoinLineupCsReq 758;
    ChangeLineupLeaderCsReq 798;
    ReplaceLineupCsReq 711;
    QuitLineupCsReq 728;

    // // Battle
    StartCocoonStageCsReq 1456;
    PveBattleResultCsReq 118;
    SceneCastSkillCsReq 1458;

    // // Teleport
    GetEnteredSceneCsReq 1469;
    GetSceneMapInfoCsReq 1451;
    EnterSceneCsReq 1425;

    // // Optional
    // GetMailCsReq 861;
    // GetGachaInfoCsReq 1961;
    // DoGachaCsReq 1991;
    // GetQuestDataCsReq 961;
    PlayerLoginFinishCsReq 76;
}
