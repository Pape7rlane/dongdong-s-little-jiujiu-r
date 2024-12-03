use std::collections::HashMap;

use crate::{
    net::tools::{FreesrData, MultiPathAvatar},
    util::cur_timestamp_ms,
};

use super::*;

pub async fn on_get_basic_info_cs_req(
    _session: &mut PlayerSession,
    _body: &GetBasicInfoCsReq,
    res: &mut GetBasicInfoScRsp,
) {
    res.player_setting_info = Some(PlayerSettingInfo::default());
    res.gender = Gender::Woman as u32;
    res.is_gender_set = true;
}

pub async fn on_player_heart_beat_cs_req(
    _session: &mut PlayerSession,
    body: &PlayerHeartBeatCsReq,
    res: &mut PlayerHeartBeatScRsp,
) {
    res.client_time_ms = body.client_time_ms;
    res.server_time_ms = cur_timestamp_ms();
    res.download_data = Some(ClientDownloadData {
        version: 51,
        time: res.server_time_ms as i64,
        data: rbase64::decode("bG9jYWwgZnVuY3Rpb24gYmV0YV90ZXh0KG9iaikKICAgIGxvY2FsIGdhbWVPYmplY3QgPSBDUy5Vbml0eUVuZ2luZS5HYW1lT2JqZWN0LkZpbmQoIlVJUm9vdC9BYm92ZURpYWxvZy9CZXRhSGludERpYWxvZyhDbG9uZSkiKQoKICAgIGlmIGdhbWVPYmplY3QgdGhlbgogICAgICAgIGxvY2FsIHRleHRDb21wb25lbnQgPSBnYW1lT2JqZWN0OkdldENvbXBvbmVudEluQ2hpbGRyZW4odHlwZW9mKENTLlJQRy5DbGllbnQuTG9jYWxpemVkVGV4dCkpCgogICAgICAgIGlmIHRleHRDb21wb25lbnQgdGhlbgogICAgICAgICAgICB0ZXh0Q29tcG9uZW50LnRleHQgPSAiUm9iaW5TUiBpcyBhIGZyZWUgYW5kIG9wZW4gc291cmNlIHNvZnR3YXJlLiBkaXNjb3JkLmdnL3JldmVyc2Vkcm9vbXMiCiAgICAgICAgZW5kCiAgICBlbHNlCiAgICBlbmQKZW5kCgpiZXRhX3RleHQoKQ==").unwrap(),
        ghkpbpdfijh: 0
    });
}

pub async fn on_player_login_finish_cs_req(
    session: &mut PlayerSession,
    _req: &PlayerLoginFinishCsReq,
    _res: &mut PlayerLoginFinishScRsp,
) -> Result<()> {
    session
        .send(ContentPackageSyncDataScNotify {
            data: Some(PackageData {
                info_list: [
                    200001, 200002, 200003, 200004, 150017, 150015, 150021, 150018, 130011, 130012,
                    130013,
                ]
                .iter()
                .map(|v| ContentInfo {
                    status: ContentPackageStatus::Finished.into(),
                    content_id: *v,
                })
                .collect(),
                ..Default::default()
            }),
        })
        .await?;

    Ok(())
}

pub async fn on_get_multi_path_avatar_info_cs_req(
    _session: &mut PlayerSession,
    _req: &GetMultiPathAvatarInfoCsReq,
    res: &mut GetMultiPathAvatarInfoScRsp,
) {
    let json = FreesrData::load().await;

    res.current_multi_path_avatar_id = HashMap::from([
        (8001, json.main_character.get_type().into()),
        (1001, json.march_type.get_type().into()),
    ]);

    res.multi_path_avatar_type_info_list = MultiPathAvatar::to_vec()
        .iter()
        .filter_map(|mp_type| {
            let avatar_info = json.avatars.get(&((*mp_type) as u32))?;
            Some(MultiPathAvatarTypeInfo {
                avatar_id: *mp_type as i32,
                rank: avatar_info.data.rank,
                equip_relic_list: json
                    .relics
                    .iter()
                    .filter(|relic| relic.equip_avatar == *mp_type as u32)
                    .map(|relic| relic.to_equipment_relic_proto())
                    .collect(),
                skilltree_list: avatar_info
                    .data
                    .skills
                    .iter()
                    .map(|(point_id, level)| AvatarSkillTree {
                        point_id: *point_id,
                        level: *level,
                    })
                    .collect(),
                path_equipment_id: json
                    .lightcones
                    .iter()
                    .find(|v| v.equip_avatar == *mp_type as u32)
                    .map(|v| v.internal_uid)
                    .unwrap_or_default(),
            })
        })
        .collect();
}
