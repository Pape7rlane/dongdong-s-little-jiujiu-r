use anyhow::Result;
use paste::paste;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tracing::Instrument;

use proto::*;
#[allow(unused_imports)]
use proto::{
    CmdActivityType::*, CmdAdventureType::*, CmdAetherDivideType::*, CmdAlleyType::*,
    CmdArchiveType::*, CmdAvatarType::*, CmdBattleCollegeType::*, CmdBattlePassType::*,
    CmdBattleType::*, CmdBoxingClubType::*, CmdChallengeType::*, CmdChatType::*,
    CmdChessRogueType::*, CmdClockParkType::*, CmdContentPackageType::*, CmdDailyActiveType::*,
    CmdDrinkMakerType::*, CmdExpeditionType::*, CmdFantasticStoryActivityType::*,
    CmdFeverTimeActivityType::*, CmdFightActivityType::*, CmdFightMathc3Type::*, CmdFightType::*,
    CmdFriendType::*, CmdGachaType::*, CmdHeartdialType::*, CmdHeliobusType::*, CmdItemType::*,
    CmdJukeboxType::*, CmdLineupType::*, CmdLobbyType::*, CmdMailType::*, CmdMapRotationType::*,
    CmdMatchThreeModuleType::*, CmdMatchType::*, CmdMessageType::*, CmdMiscModuleType::*,
    CmdMissionType::*, CmdMonopolyType::*, CmdMultiplayerType::*, CmdMultipleDropType::*,
    CmdMuseumType::*, CmdOfferingType::*, CmdPamMissionType::*, CmdPhoneType::*,
    CmdPlayerBoardType::*, CmdPlayerReturnType::*, CmdPlayerSync::*, CmdPlayerType::*,
    CmdPlotType::*, CmdPunkLordType::*, CmdQuestType::*, CmdRaidCollectionType::*, CmdRaidType::*,
    CmdRedDotType::*, CmdReplayType::*, CmdRndOptionType::*, CmdRogueCommonType::*,
    CmdRogueEndless::*, CmdRogueModifierType::*, CmdRogueTournType::*, CmdRogueType::*,
    CmdRollShopType::*, CmdSceneType::*, CmdServerPrefsType::*, CmdShopType::*, CmdSpaceZooType::*,
    CmdStarFightType::*, CmdStoryLineType::*, CmdStrongChallengeActivityType::*,
    CmdTalkRewardType::*, CmdTelevisionActivityType::*, CmdTextJoinType::*, CmdTrainVisitorType::*,
    CmdTreasureDungeonType::*, CmdTutorialType::*, CmdWaypointType::*, CmdWolfBroType::*,
};

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
    ($($name:tt;)*) => {
        pub trait CommandHandler {
            $(
                paste! {
                    async fn [<on_$name:snake _cs_req>](session: &mut PlayerSession, request: &[<$name CsReq>]) -> Result<()> {
                        let mut response = proto::[<$name ScRsp>]::default();
                        let _ = [<on_$name:snake _cs_req>](session, request, &mut response).await;
                        session.send(response).await?;

                        Ok(())
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
                        cmd_type if cmd_type == paste! { [<Cmd$name CsReq>] as u16 }  => {
                            let body = paste! { proto::[<$name CsReq>]::decode(&mut &payload[..])? };
                            paste! {
                                Self::[<on_$name:snake _cs_req>](session, &body)
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
    PlayerGetToken;
    PlayerLogin;
    GetMissionStatus;
    GetBasicInfo;
    GetMultiPathAvatarInfo;
    GetAvatarData;
    GetAllLineupData;
    GetCurLineupData;
    GetCurSceneInfo;
    PlayerHeartBeat;


    // Entity move (dummy!)
    SceneEntityMove;

    // Inventory (dummy!)
    GetBag;
    GetArchiveData;
    DressAvatar;
    TakeOffEquipment;
    DressRelicAvatar;
    TakeOffRelic;

    // Chat (dummy!)
    SendMsg;
    GetPrivateChatHistory;
    GetFriendListInfo;
    GetFriendLoginInfo;

    // In-game lineup
    JoinLineup;
    ChangeLineupLeader;
    ReplaceLineup;
    QuitLineup;

    // Battle
    StartCocoonStage;
    PveBattleResult;
    SceneCastSkill;

    // Teleport
    GetEnteredScene;
    GetSceneMapInfo;
    EnterScene;

    // Optional
    GetMail;
    GetGachaInfo;
    DoGacha;
    PlayerLoginFinish;
}
