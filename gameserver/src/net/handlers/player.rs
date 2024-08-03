use std::collections::HashMap;

use crate::{net::tools::FreesrData, util};

use super::*;

pub async fn on_get_basic_info_cs_req(
    session: &mut PlayerSession,
    _body: &GetBasicInfoCsReq,
) -> Result<()> {
    session
        .send(
            CMD_GET_BASIC_INFO_SC_RSP,
            GetBasicInfoScRsp {
                retcode: 0,
                player_setting_info: Some(PlayerSettingInfo::default()),
                gender: Gender::Woman as u32,
                is_gender_set: true,
                ..Default::default()
            },
        )
        .await
}

pub async fn on_player_heart_beat_cs_req(
    session: &mut PlayerSession,
    body: &PlayerHeartBeatCsReq,
) -> Result<()> {
    session
        .send(
            CMD_PLAYER_HEART_BEAT_SC_RSP,
            PlayerHeartBeatScRsp {
                retcode: 0,
                client_time_ms: body.client_time_ms,
                server_time_ms: util::cur_timestamp_ms(),
                download_data: Some(ClientDownloadData {
                    version: 51,
                    time: util::cur_timestamp_ms() as i64,
                    data: rbase64::decode("bG9jYWwgZnVuY3Rpb24gYmV0YV90ZXh0KG9iaikKICAgIGxvY2FsIGdhbWVPYmplY3QgPSBDUy5Vbml0eUVuZ2luZS5HYW1lT2JqZWN0LkZpbmQoIlVJUm9vdC9BYm92ZURpYWxvZy9CZXRhSGludERpYWxvZyhDbG9uZSkiKQoKICAgIGlmIGdhbWVPYmplY3QgdGhlbgogICAgICAgIGxvY2FsIHRleHRDb21wb25lbnQgPSBnYW1lT2JqZWN0OkdldENvbXBvbmVudEluQ2hpbGRyZW4odHlwZW9mKENTLlJQRy5DbGllbnQuTG9jYWxpemVkVGV4dCkpCgogICAgICAgIGlmIHRleHRDb21wb25lbnQgdGhlbgogICAgICAgICAgICB0ZXh0Q29tcG9uZW50LnRleHQgPSAiUm9iaW5TUiBpcyBhIGZyZWUgYW5kIG9wZW4gc291cmNlIHNvZnR3YXJlLiBkaXNjb3JkLmdnL3JldmVyc2Vkcm9vbXMiCiAgICAgICAgZW5kCiAgICBlbHNlCiAgICBlbmQKZW5kCgpiZXRhX3RleHQoKQ==").unwrap()
                }),
            },
        )
        .await
}

pub type PlayerLoginFinishCsReq = Dummy;

pub async fn on_player_login_finish_cs_req(
    session: &mut PlayerSession,
    _: &PlayerLoginFinishCsReq,
) -> Result<()> {
    session
        .send(CMD_PLAYER_LOGIN_FINISH_SC_RSP, Dummy::default())
        .await?;

    session
        .send(
            CMD_CONTENT_PACKAGE_SYNC_DATA_SC_NOTIFY,
            ContentPackageSyncDataScNotify {
                data: Some(PackageData {
                    info_list: vec![
                        ContentInfo {
                            status: ContentPackageStatus::Finished.into(),
                            content_id: 200001,
                        },
                        ContentInfo {
                            status: ContentPackageStatus::Finished.into(),
                            content_id: 200002,
                        },
                        ContentInfo {
                            status: ContentPackageStatus::Finished.into(),
                            content_id: 200003,
                        },
                        ContentInfo {
                            status: ContentPackageStatus::Finished.into(),
                            content_id: 150017,
                        },
                        ContentInfo {
                            status: ContentPackageStatus::Finished.into(),
                            content_id: 150015,
                        },
                    ],
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}

pub async fn on_get_multi_path_avatar_info_cs_req(
    session: &mut PlayerSession,
    _: &GetMultiPathAvatarInfoCsReq,
) -> Result<()> {
    let json = FreesrData::load().await;

    session
        .send(
            CMD_GET_MULTI_PATH_AVATAR_INFO_SC_RSP,
            GetMultiPathAvatarInfoScRsp {
                retcode: 0,
                multi_path_avatar_type_info_list: vec![
                    MultiPathAvatarTypeInfo {
                        avatar_id: json.main_character as i32,
                        rank: 6,
                        equip_relic_list: json
                            .relics
                            .iter()
                            .filter(|v| v.equip_avatar == json.main_character as u32)
                            .map(|v| v.to_equipment_relic_proto())
                            .collect(),
                        skilltree_list: (1..=4)
                            .map(|v| AvatarSkillTree {
                                level: 1,
                                point_id: (json.main_character as u32) * 1000 + v,
                            })
                            .collect(),
                        path_equipment_id: json
                            .lightcones
                            .iter()
                            .find(|v| v.equip_avatar == json.main_character as u32)
                            .map(|v| v.internal_uid)
                            .unwrap_or_default(),
                    },
                    MultiPathAvatarTypeInfo {
                        avatar_id: json.march_type as i32,
                        rank: 6,
                        equip_relic_list: json
                            .relics
                            .iter()
                            .filter(|v| v.equip_avatar == json.march_type as u32)
                            .map(|v| v.to_equipment_relic_proto())
                            .collect(),
                        skilltree_list: (1..=4)
                            .map(|v| AvatarSkillTree {
                                level: 1,
                                point_id: (json.march_type as u32) * 1000 + v,
                            })
                            .collect(),
                        path_equipment_id: json
                            .lightcones
                            .iter()
                            .find(|v| v.equip_avatar == json.march_type as u32)
                            .map(|v| v.internal_uid)
                            .unwrap_or_default(),
                    },
                ],
                current_multi_path_avatar_id: HashMap::from([
                    (8001, json.main_character.get_type().into()),
                    (1001, json.march_type.get_type().into()),
                ]),
                ..Default::default()
            },
        )
        .await
}
