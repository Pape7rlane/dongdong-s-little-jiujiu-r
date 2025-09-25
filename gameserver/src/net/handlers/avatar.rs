use std::collections::HashMap;

use super::*;

pub static UNLOCKED_AVATARS: [u32; 73] = [
    1002, 1003, 1004, 1005, 1006, 1008, 1009, 1013, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
    1109, 1110, 1111, 1112, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1208, 1209, 1210, 1211, 1212,
    1213, 1214, 1215, 1217, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1308, 1309, 1312, 1315, 1310,
    1314, 1218, 1221, 1220, 1222, 1223, 1317, 1313, 1225, 1402, 1401, 1404, 1403, 1405, 1407, 1406,
    1409, 1014, 1015, 1408, 1410, 1412, 1413, 1414, 1415
];

pub async fn on_get_avatar_data_cs_req(
    session: &mut PlayerSession,
    body: &GetAvatarDataCsReq,
    res: &mut GetAvatarDataScRsp,
) {
    let Some(json) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    // TODO: HARDCODED
    let mc_ids = if json.main_character.get_gender() == Gender::Man {
        [8001, 8003, 8005, 8007]
    } else {
        [8002, 8004, 8006, 8008]
    };

    let march_ids = [1001, 1224];

    res.is_get_all = body.is_get_all;
    res.avatar_list = UNLOCKED_AVATARS
        .into_iter()
        .chain(mc_ids.iter().copied())
        .chain(march_ids.iter().copied())
        .map(|id| {
            json.avatars
                .get(&id)
                .map(|v| {
                    v.to_avatar_proto(
                        json.lightcones.iter().find(|v| v.equip_avatar == id),
                        json.relics
                            .iter()
                            .filter(|v| v.equip_avatar == id)
                            .collect(),
                    )
                })
                .unwrap_or(Avatar {
                    base_avatar_id: id,
                    level: 80,
                    promotion: 6,
                    rank: 6,
                    skilltree_list: (1..=4)
                        .map(|m| AvatarSkillTree {
                            point_id: id * 1000 + m,
                            level: 1,
                        })
                        .collect(),
                    first_met_timestamp: 1712924677,
                    ..Default::default()
                })
        })
        .collect();

    res.current_multi_path_avatar_id = HashMap::from([
        (8001, json.main_character.get_type().into()),
        (1001, json.march_type.get_type().into()),
    ]);

    res.multi_path_avatar_type_info_list = json.get_multi_path_info();
}

pub async fn on_set_avatar_enhanced_id_cs_req(
    _session: &mut PlayerSession,
    _req: &SetAvatarEnhancedIdCsReq,
    _res: &mut SetAvatarEnhancedIdScRsp,
) {
    // let Some(json) = session.json_data.get_mut() else {
    //     return;
    // };
    // let Some(avatar) = json.avatars.get_mut(&req.avatar_id) else {
    //     return;
    // };
    // avatar.enhanced_id = if req.enhanced_id == 0 {
    //     Option::<u32>::None
    // } else {
    //     Some(req.enhanced_id)
    // };
    // res.growth_avatar_id = avatar.avatar_id;
    // res.unk_enhanced_id = req.enhanced_id;
}
