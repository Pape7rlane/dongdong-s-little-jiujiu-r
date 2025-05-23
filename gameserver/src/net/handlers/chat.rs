use common::structs::MultiPathAvatar;

use crate::{net::PlayerSession, util::cur_timestamp_ms};

use super::*;

const SERVER_UID: u32 = 727;
const SERVER_HEAD_ICON: u32 = 201402;
const SERVER_CHAT_BUBBLE_ID: u32 = 220005;
const SERVER_CHAT_HISTORY: [&str; 5] = [
    "'sync' to synchronize stats between json and in-game view",
    "'mc {mc_id}' mc_id can be set from 8001 to 8008",
    "'march {march_id}' march_id can be set 1001 or 1224",
    "available commands:",
    "visit srtools.neonteam.dev to configure the PS! (you configure relics, equipment, monsters from there)",
];

pub async fn on_get_friend_login_info_cs_req(
    _session: &mut PlayerSession,
    _req: &GetFriendLoginInfoCsReq,
    res: &mut GetFriendLoginInfoScRsp,
) {
    res.friend_uid_list = vec![SERVER_UID];
}

pub async fn on_get_friend_list_info_cs_req(
    _session: &mut PlayerSession,
    _req: &GetFriendListInfoCsReq,
    res: &mut GetFriendListInfoScRsp,
) {
    res.friend_list = vec![FriendListInfo {
        friend_name: String::from("RobinSR"),
        simple_info: Some(SimpleInfo {
            uid: SERVER_UID,
            platform_type: PlatformType::Pc.into(),
            online_status: FriendOnlineStatus::Online.into(),
            head_icon: SERVER_HEAD_ICON,
            chat_bubble_id: SERVER_CHAT_BUBBLE_ID,
            level: 70,
            nickname: String::from("Server"),
            signature: String::from("omg"),
            ..Default::default()
        }),
        is_marked: true,
        sent_time: 0,
        ..Default::default()
    }];
}

pub async fn on_get_private_chat_history_cs_req(
    _session: &mut PlayerSession,
    req: &GetPrivateChatHistoryCsReq,
    res: &mut GetPrivateChatHistoryScRsp,
) {
    let cur_time = cur_timestamp_ms();
    res.chat_list = SERVER_CHAT_HISTORY
        .iter()
        .map(|text| Chat {
            msg_type: MsgType::CustomText.into(),
            sent_time: cur_time,
            text: String::from(*text),
            sender_uid: SERVER_UID,
            ..Default::default()
        })
        .collect();
    res.to_uid = req.to_uid;
    res.from_uid = SERVER_UID;
}

pub async fn on_send_msg_cs_req(
    session: &mut PlayerSession,
    body: &SendMsgCsReq,
    _res: &mut SendMsgScRsp,
) {
    let Some(json) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    if let Some((cmd, args)) = parse_command(&body.text) {
        match cmd {
            "sync" => {
                session.sync_player().await;
                session
                    .send(RevcMsgScNotify {
                        msg_type: body.msg_type,
                        text: String::from("Inventory Synced"),
                        emote: body.emote,
                        from_uid: SERVER_UID,
                        to_uid: 25,
                        chat_type: body.chat_type,
                        hnbepabnbng: body.hnbepabnbng,
                    })
                    .await
                    .unwrap();
            }
            "mc" => {
                let mc = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.main_character as u32),
                );

                json.main_character = mc;
                json.save_persistent().await;

                session
                    .send(AvatarPathChangedNotify {
                        base_avatar_id: 8001,
                        cur_multi_path_avatar_type: mc as i32,
                    })
                    .await
                    .unwrap();

                session.sync_player().await;

                session
                    .send(RevcMsgScNotify {
                        msg_type: body.msg_type,
                        text: format!("Success change mc to {:#?}", mc),
                        emote: body.emote,
                        from_uid: SERVER_UID,
                        to_uid: 25,
                        chat_type: body.chat_type,
                        hnbepabnbng: body.hnbepabnbng,
                    })
                    .await
                    .unwrap();
            }
            "march" => {
                let mut march_type = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.march_type as u32),
                );

                if march_type != MultiPathAvatar::MarchPreservation
                    && march_type != MultiPathAvatar::MarchHunt
                {
                    march_type = MultiPathAvatar::MarchHunt
                }

                json.march_type = march_type;
                json.save_persistent().await;

                session
                    .send(AvatarPathChangedNotify {
                        base_avatar_id: 1001,
                        cur_multi_path_avatar_type: march_type as i32,
                    })
                    .await
                    .unwrap();

                session
                    .send(RevcMsgScNotify {
                        msg_type: body.msg_type,
                        text: format!("Success change march to {:#?}", march_type),
                        emote: body.emote,
                        from_uid: SERVER_UID,
                        to_uid: 25,
                        chat_type: body.chat_type,
                        hnbepabnbng: body.hnbepabnbng,
                    })
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

fn parse_command(command: &str) -> Option<(&str, Vec<&str>)> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Option::None;
    }

    Some((parts[0], parts[1..].to_vec()))
}
