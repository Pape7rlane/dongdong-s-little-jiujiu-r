use crate::{
    net::{tools::JsonData, PlayerSession},
    util::cur_timestamp_ms,
};

use super::*;

pub async fn on_get_friend_list_info_cs_req(
    session: &mut PlayerSession,
    _: &GetFriendListInfoCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_FRIEND_LIST_INFO_SC_RSP,
            GetFriendListInfoScRsp {
                friend_list: vec![FriendListInfo {
                    playing_state: PlayingState::None.into(),
                    efhdeanophb: Some(Oohgaobiikp::default()),
                    kkmacpkifkp: String::from("RobinSR"),
                    simple_info: Some(SimpleInfo {
                        uid: 727,
                        platform_type: 3,
                        online_status: 1,
                        head_icon: 201008,
                        chat_bubble_id: 220005,
                        level: 70,
                        nickname: String::from("Server"),
                        assist_simple_info: vec![AssistSimpleInfo {
                            avatar_id: 1008,
                            level: 70,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    is_marked: true,
                }],
                ..Default::default()
            },
        )
        .await
}

pub async fn on_get_private_chat_history_cs_req(
    session: &mut PlayerSession,
    _: &GetPrivateChatHistoryCsReq,
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
                        text: "available command:".to_string(),
                        sender_uid: 727,
                        ..Default::default()
                    },
                ],
                sender_id: 727, // from
                to_uid: 1337,   // to
                ..Default::default()
            },
        )
        .await
}

pub async fn on_send_msg_cs_req(session: &mut PlayerSession, body: &SendMsgCsReq) -> Result<()> {
    let json = JsonData::load().await;

    if let Some((cmd, _args)) = parse_command(&body.text) {
        match cmd {
            "sync" => {
                sync_player(session, json).await?;
                session
                    .send(
                        CMD_REVC_MSG_SC_NOTIFY,
                        RevcMsgScNotify {
                            enlmbccjfbg: body.enlmbccjfbg.clone(),
                            msg_type: body.msg_type,
                            text: String::from("Inventory Synced"),
                            emote: body.emote,
                            from_uid: 727, // from
                            to_uid: 1337,  // to
                            chat_type: body.chat_type,
                        },
                    )
                    .await?;
            }
            "mc" => {
                // let mc = MainCharacter::from(
                //     args.first()
                //         .unwrap_or(&"")
                //         .parse::<u32>()
                //         .unwrap_or(json.main_character as u32),
                // );

                // json.main_character = mc;
                // json.save().await;

                // session
                //     .send(
                //         CMD_GET_HERO_BASIC_TYPE_INFO_SC_RSP,
                //         GetHeroBasicTypeInfoScRsp {
                //             retcode: 0,
                //             gender: mc.get_gender().into(),
                //             cur_basic_type: mc.get_type().into(),
                //             basic_type_info_list: vec![HeroBasicTypeInfo {
                //                 basic_type: mc.get_type().into(),
                //                 ..Default::default()
                //             }],
                //             ..Default::default()
                //         },
                //     )
                //     .await?;

                // sync_player(session, json).await?;

                // session
                //     .send(
                //         CMD_REVC_MSG_SC_NOTIFY,
                //         Klonpheafip {
                //             ggadmjhlomj: body.ggadmjhlomj.clone(),
                //             kokadficdfb: body.kokadficdfb,
                //             ajlhdpcjand: format!("Set MC to: {mc:#?}"),
                //             mgmicgabebd: body.mgmicgabebd,
                //             ghojifhngmc: 727,
                //             cmmildghfnl: 1337,
                //             nmfepfoojic: body.nmfepfoojic,
                //         },
                //     )
                //     .await?;
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

async fn sync_player(session: &mut PlayerSession, json: JsonData) -> Result<()> {
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
