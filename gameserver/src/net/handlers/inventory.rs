use proto::{get_big_data_all_recommend_sc_rsp::RecommendType, *};

use crate::net::PlayerSession;

use super::UNLOCKED_AVATARS;

pub async fn on_get_bag_cs_req(
    session: &mut PlayerSession,
    _req: &GetBagCsReq,
    res: &mut GetBagScRsp,
) {
    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    res.equipment_list = player
        .lightcones
        .iter()
        .map(|v| v.to_equipment_proto())
        .collect();
    res.relic_list = player.relics.iter().map(|v| v.to_relic_proto()).collect();
    res.material_list = vec![
        Material {
            tid: 101, // Normal Pass
            num: 999999,
            ..Default::default()
        },
        Material {
            tid: 102, // Special Pass
            num: 999999,
            ..Default::default()
        },
    ];
}

pub async fn on_get_archive_data_cs_req(
    _session: &mut PlayerSession,
    _: &GetArchiveDataCsReq,
    res: &mut GetArchiveDataScRsp,
) {
    res.archive_data = Some(ArchiveData::default());
}

pub async fn on_dress_relic_avatar_cs_req(
    _session: &mut PlayerSession,
    _req: &DressRelicAvatarCsReq,
    _res: &mut DressRelicAvatarScRsp,
) {
}

pub async fn on_take_off_relic_cs_req(
    _session: &mut PlayerSession,
    _req: &TakeOffRelicCsReq,
    _res: &mut TakeOffRelicScRsp,
) {
}

pub async fn on_dress_avatar_cs_req(
    _session: &mut PlayerSession,
    _req: &DressAvatarCsReq,
    _res: &mut DressAvatarScRsp,
) {
}

pub async fn on_take_off_equipment_cs_req(
    _session: &mut PlayerSession,
    _req: &TakeOffEquipmentCsReq,
    _res: &mut TakeOffEquipmentScRsp,
) {
}

pub async fn on_get_big_data_all_recommend_cs_req(
    _: &mut PlayerSession,
    req: &GetBigDataAllRecommendCsReq,
    res: &mut GetBigDataAllRecommendScRsp,
) {
    res.big_data_recommend_type = req.big_data_recommend_type;

    match req.big_data_recommend_type() {
        BigDataRecommendType::RelicAvatar => {
            res.recommend_type = Some(RecommendType::RelicAvatar(BigDataRecommendRelicAvatar {
                recommended_avatar_info_list: UNLOCKED_AVATARS
                    .into_iter()
                    .map(|recommend_avatar_id| RecomendedAvatarInfo {
                        avatar_id_list: vec![],
                        recommend_avatar_id,
                        relic_set_id: 0,
                    })
                    .collect(),
            }))
        }
        BigDataRecommendType::AvatarRelic => {
            res.recommend_type = Some(RecommendType::AvatarRelic(BigDataRecommendAvatarRelic {
                recomended_relic_info_list: UNLOCKED_AVATARS
                    .into_iter()
                    .map(|avatar_id| RecommendedRelicInfo {
                        avatar_id,
                        ..Default::default()
                    })
                    .collect(),
            }))
        }
        _ => {}
    }
}
