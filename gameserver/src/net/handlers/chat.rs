use crate::{
    net::{
        tools::{FreesrData, MultiPathAvatar},
        PlayerSession,
    },
    util::cur_timestamp_ms,
};

use super::*;

pub async fn on_get_friend_login_info_cs_req(
    session: &mut PlayerSession,
    _: &GetFriendLoginInfoCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_FRIEND_LOGIN_INFO_SC_RSP,
            GetFriendLoginInfoScRsp {
                mnacbpjeche: vec![727],
                ..Default::default()
            },
        )
        .await
}

pub async fn on_get_friend_list_info_cs_req(
    session: &mut PlayerSession,
    _: &GetFriendListInfoCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_FRIEND_LIST_INFO_SC_RSP,
            GetFriendListInfoScRsp {
                friend_list: vec![FriendListInfo {
                    friend_name: String::from("RobinSR"),
                    simple_info: Some(SimpleInfo {
                        uid: 727,
                        platform_type: 3,
                        online_status: 1,
                        head_icon: 201008,
                        chat_bubble_id: 220005,
                        level: 70,
                        nickname: String::from("Server"),
                        signature: String::from("omg"),
                        ..Default::default()
                    }),
                    is_marked: true,
                    sent_time: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .await
}

pub async fn on_get_private_chat_history_cs_req(
    session: &mut PlayerSession,
    req: &GetPrivateChatHistoryCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_PRIVATE_CHAT_HISTORY_SC_RSP,
            GetPrivateChatHistoryScRsp {
                chat_list: vec![
                    Chat {
                        msg_type: MsgType::CustomText.into(),
                        sent_time: cur_timestamp_ms(),
                        text: "'sync'".to_string(),
                        sender_uid: 727,
                        ..Default::default()
                    },
                    Chat {
                        msg_type: MsgType::CustomText.into(),
                        sent_time: cur_timestamp_ms(),
                        text: "'mc {mc_id}' mc_id can be set from 8001 to 8006".to_string(),
                        sender_uid: 727,
                        ..Default::default()
                    },
                    Chat {
                        msg_type: MsgType::CustomText.into(),
                        sent_time: cur_timestamp_ms(),
                        text: "'march {march_id}' march_id can be set 1001 or 1224".to_string(),
                        sender_uid: 727,
                        ..Default::default()
                    },
                    Chat {
                        msg_type: MsgType::CustomText.into(),
                        sent_time: cur_timestamp_ms(),
                        text: "available command:".to_string(),
                        sender_uid: 727,
                        ..Default::default()
                    },
                ],
                sender_id: 0,       // from
                to_uid: req.to_uid, // to
                ..Default::default()
            },
        )
        .await
}

pub async fn on_send_msg_cs_req(session: &mut PlayerSession, body: &SendMsgCsReq) -> Result<()> {
    let mut json = FreesrData::load().await;

    if let Some((cmd, args)) = parse_command(&body.text) {
        match cmd {
            "sync" => {
                sync_player(session, json).await?;
                session
                    .send(
                        CMD_REVC_MSG_SC_NOTIFY,
                        RevcMsgScNotify {
                            msg_type: body.msg_type,
                            text: String::from("Inventory Synced"),
                            emote: body.emote,
                            from_uid: 727, // from
                            to_uid: 25,    // to
                            chat_type: body.chat_type,
                        },
                    )
                    .await?;
            }
            "mc" => {
                let mc = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.main_character as u32),
                );

                json.main_character = mc;
                json.save().await;

                session
                    .send(
                        CMD_AVATAR_PATH_CHANGED_NOTIFY,
                        AvatarPathChangedNotify {
                            base_avatar_id: 8001,
                            cur_multi_path_avatar_type: mc as i32,
                        },
                    )
                    .await?;

                session
                    .send(
                        CMD_REVC_MSG_SC_NOTIFY,
                        RevcMsgScNotify {
                            msg_type: body.msg_type,
                            text: format!("Success change mc to {:#?}", mc),
                            emote: body.emote,
                            from_uid: 727, // from
                            to_uid: 25,    // to
                            chat_type: body.chat_type,
                        },
                    )
                    .await?;
            }
            "march" => {
                let march_type = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.march_type as u32),
                );

                json.march_type = march_type;
                json.save().await;

                session
                    .send(
                        CMD_AVATAR_PATH_CHANGED_NOTIFY,
                        AvatarPathChangedNotify {
                            base_avatar_id: 1001,
                            cur_multi_path_avatar_type: march_type as i32,
                        },
                    )
                    .await?;

                session
                    .send(
                        CMD_REVC_MSG_SC_NOTIFY,
                        RevcMsgScNotify {
                            msg_type: body.msg_type,
                            text: format!("Success change march to {:#?}", march_type),
                            emote: body.emote,
                            from_uid: 727, // from
                            to_uid: 25,    // to
                            chat_type: body.chat_type,
                        },
                    )
                    .await?;
            }
            _ => {}
        }
    }

    session
        .send(
            CMD_SEND_MSG_SC_RSP,
            SendMsgScRsp {
                retcode: 0,
                end_time: 0,
            },
        )
        .await
}

fn parse_command(command: &str) -> Option<(&str, Vec<&str>)> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Option::None;
    }

    Some((parts[0], parts[1..].to_vec()))
}

async fn sync_player(session: &mut PlayerSession, json: FreesrData) -> Result<()> {
    session
        .send(
            CMD_PLAYER_SYNC_SC_NOTIFY,
            PlayerSyncScNotify {
                del_equipment_list: (2000..3500).collect(),
                del_relic_list: (1..2000).collect(),
                ..Default::default()
            },
        )
        .await?;

    session
        .send(
            CMD_PLAYER_SYNC_SC_NOTIFY,
            PlayerSyncScNotify {
                avatar_sync: Some(AvatarSync {
                    avatar_list: json
                        .avatars
                        .values()
                        .map(|avatar| avatar.to_avatar_proto(Option::None, vec![]))
                        .collect::<Vec<_>>(),
                }),
                ..Default::default()
            },
        )
        .await?;

    session
        .send(
            CMD_PLAYER_SYNC_SC_NOTIFY,
            PlayerSyncScNotify {
                relic_list: json.relics.iter().map(|v| v.to_relic_proto()).collect(),
                equipment_list: json
                    .lightcones
                    .iter()
                    .map(|v| v.to_equipment_proto())
                    .collect(),
                ..Default::default()
            },
        )
        .await?;

    session
        .send(
            CMD_PLAYER_SYNC_SC_NOTIFY,
            PlayerSyncScNotify {
                avatar_sync: Some(AvatarSync {
                    avatar_list: json
                        .avatars
                        .values()
                        .map(|avatar| {
                            avatar.to_avatar_proto(
                                json.lightcones
                                    .iter()
                                    .find(|v| v.equip_avatar == avatar.avatar_id),
                                json.relics
                                    .iter()
                                    .filter(|v| v.equip_avatar == avatar.avatar_id)
                                    .collect(),
                            )
                        })
                        .collect(),
                }),
                ..Default::default()
            },
        )
        .await
}
