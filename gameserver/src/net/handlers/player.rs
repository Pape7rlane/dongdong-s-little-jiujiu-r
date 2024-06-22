use std::collections::HashMap;

use crate::{
    net::tools::JsonData,
    util::{self, cur_timestamp_ms},
};

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
                ..Default::default()
            },
        )
        .await
}

// pub async fn on_get_hero_basic_type_info_cs_req(
//     session: &mut PlayerSession,
//     _body: &GetHeroBasicTypeInfoCsReq,
// ) -> Result<()> {
//     let mc = JsonData::load().await.main_character;
//     session
//         .send(
//             CMD_GET_HERO_BASIC_TYPE_INFO_SC_RSP,
//             GetHeroBasicTypeInfoScRsp {
//                 retcode: 0,
//                 gender: mc.get_gender().into(),
//                 cur_basic_type: mc.get_type().into(),
//                 basic_type_info_list: vec![HeroBasicTypeInfo {
//                     basic_type: mc.get_type().into(),
//                     ..Default::default()
//                 }],
//                 ..Default::default()
//             },
//         )
//         .await
// }

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
                    data: rbase64::decode("G0x1YVMBGZMNChoKBAQICHhWAAAAAAAAAAAAAAAod0ABKEBDOlxVc2Vyc1x4ZW9uZGV2XERvd25sb2Fkc1xyYWJzdHZvLmx1YQAAAAAAAAAAAAEEEAAAACQAQAApQEAAKYBAACnAQABWAAEALIAAAR1AQQCkgEEA5ABAAOnAwQHpAMIB6UDCAawAAAEsgAAAH8BChRkAgAAMAAAABANDUwQMVW5pdHlFbmdpbmUEC0dhbWVPYmplY3QEBUZpbmQEKVVJUm9vdC9BYm92ZURpYWxvZy9CZXRhSGludERpYWxvZyhDbG9uZSkEF0dldENvbXBvbmVudEluQ2hpbGRyZW4EB3R5cGVvZgQEUlBHBAdDbGllbnQEDkxvY2FsaXplZFRleHQEBXRleHQURVJvYmluU1IgaXMgYSBmcmVlIGFuZCBvcGVuIHNvdXJjZSBzb2Z0d2FyZS4gZGlzY29yZC5nZy9yZXZlcnNlZHJvb21zAQAAAAEAAAAAABAAAAABAAAAAQAAAAEAAAABAAAAAQAAAAEAAAABAAAAAQAAAAEAAAABAAAAAQAAAAEAAAABAAAAAQAAAAEAAAABAAAAAAAAAAEAAAAFX0VOVg==").unwrap()
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
        .send(CMD_PLAYER_LOGIN_FINISH_SC_RSP, Dummy {})
        .await?;
    session
        .send(CMD_CONTENT_PACKAGE_UNLOCK_SC_RSP, Dummy {})
        .await?;
    session
        .send(CMD_CONTENT_PACKAGE_GET_DATA_SC_RSP, Dummy {})
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
                    fnmhgeaflmn: 0,
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
    session
        .send(
            CMD_GET_MULTI_PATH_AVATAR_INFO_SC_RSP,
            GetMultiPathAvatarInfoScRsp {
                retcode: 0,
                multi_path_avatar_type_info_list: vec![
                    MultiPathAvatarTypeInfo {
                        multi_path_avatar_type: MultiPathAvatarType::GirlShamanType.into(),
                        rank: 6,
                        equip_relic_list: vec![],
                        skilltree_list: vec![
                            AvatarSkillTree {
                                level: 1,
                                point_id: 8006001,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 8006002,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 8006003,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 8006004,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 8006001,
                            },
                        ],
                        all_path_special_skilltree_list: vec![],
                        path_equipment_id: 0,
                        all_path_unlocked_special_point_id_list: vec![],
                    },
                    MultiPathAvatarTypeInfo {
                        multi_path_avatar_type: MultiPathAvatarType::Mar7thKnightType.into(),
                        rank: 6,
                        equip_relic_list: vec![],
                        skilltree_list: vec![
                            AvatarSkillTree {
                                level: 1,
                                point_id: 1224001,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 1224002,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 1224003,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 1224004,
                            },
                            AvatarSkillTree {
                                level: 1,
                                point_id: 1224005,
                            },
                        ],
                        all_path_special_skilltree_list: vec![],
                        path_equipment_id: 1224,
                        all_path_unlocked_special_point_id_list: vec![],
                    },
                ],
                gmhdhimdkfi: vec![],
                current_multi_path_avatar_id: HashMap::from([
                    (8001, MultiPathAvatarType::GirlShamanType.into()),
                    (1001, MultiPathAvatarType::Mar7thRogueType.into()),
                ]),
            },
        )
        .await
}
