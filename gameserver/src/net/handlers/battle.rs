use std::collections::HashMap;

use rand::Rng;

use crate::net::tools::{self, BattleType, Monster};

use super::*;

pub async fn on_start_cocoon_stage_cs_req(
    session: &mut PlayerSession,
    body: &StartCocoonStageCsReq,
) -> Result<()> {
    let battle_info = create_battle_info().await;

    let rsp = StartCocoonStageScRsp {
        retcode: 0,
        prop_entity_id: body.prop_entity_id,
        cocoon_id: body.cocoon_id,
        wave: body.wave,
        battle_info: Some(battle_info),
    };

    session.send(CMD_START_COCOON_STAGE_SC_RSP, rsp).await
}

pub async fn on_pve_battle_result_cs_req(
    session: &mut PlayerSession,
    body: &PveBattleResultCsReq,
) -> Result<()> {
    session
        .send(
            CMD_PVE_BATTLE_RESULT_SC_RSP,
            PveBattleResultScRsp {
                retcode: 0,
                end_status: body.end_status,
                battle_id: body.battle_id,
                ..Default::default()
            },
        )
        .await
}

pub async fn on_scene_cast_skill_cs_req(
    session: &mut PlayerSession,
    request: &SceneCastSkillCsReq,
) -> Result<()> {
    let battle_info = create_battle_info().await;

    let mut resp = SceneCastSkillScRsp {
        attacked_group_id: request.attacked_group_id,
        retcode: 0,
        ..Default::default()
    };

    let targets = request
        .assist_monster_entity_id_list
        .iter()
        .filter(|id| **id > 30_000 || **id < 1_000)
        .collect::<Vec<_>>();

    if targets.is_empty() {
        return session.send(CMD_SCENE_CAST_SKILL_SC_RSP, resp).await;
    }

    resp.battle_info = Some(battle_info);

    session.send(CMD_SCENE_CAST_SKILL_SC_RSP, resp).await
}

async fn create_battle_info() -> SceneBattleInfo {
    let player = tools::FreesrData::load().await;

    let mut battle_info = SceneBattleInfo {
        stage_id: player.battle_config.stage_id,
        logic_random_seed: rand::thread_rng().gen::<u32>(),
        battle_id: 1,
        rounds_limit: player.battle_config.cycle_count,
        ..Default::default()
    };

    // avatars
    for (i, avatar_id) in player.lineups.iter() {
        if let Some(avatar) = player.avatars.get(avatar_id) {
            let (battle_avatar, techs) = avatar.to_battle_avatar_proto(
                *i,
                player
                    .lightcones
                    .iter()
                    .find(|v| v.equip_avatar == avatar.avatar_id),
                player
                    .relics
                    .iter()
                    .filter(|v| v.equip_avatar == avatar.avatar_id)
                    .collect::<Vec<_>>(),
            );
            for tech in techs {
                battle_info.buff_list.push(tech);
            }
            battle_info.battle_avatar_list.push(battle_avatar);

            // hardcoded march
            if avatar.avatar_id == 1224 {
                let buffs = BattleBuff {
                    id: 122401,
                    level: 3,
                    wave_flag: 0xffffffff,
                    owner_id: *i,
                    dynamic_values: HashMap::from([
                        (String::from("#ADF_1"), 3f32),
                        (String::from("#ADF_2"), 3f32),
                    ]),
                    target_index_list: vec![0],
                };

                battle_info.buff_list.push(buffs);
            }
        };
    }

    // custom stats for avatars
    for stat in &player.battle_config.custom_stats {
        for avatar in &mut battle_info.battle_avatar_list {
            if avatar.relic_list.is_empty() {
                avatar.relic_list.push(BattleRelic {
                    id: 61011,
                    main_affix_id: 1,
                    level: 1,
                    ..Default::default()
                })
            }

            if let Some(sub_affix) = avatar.relic_list[0]
                .sub_affix_list
                .iter_mut()
                .find(|v| v.affix_id == stat.sub_affix_id)
            {
                sub_affix.cnt = stat.count;
            } else {
                avatar.relic_list[0].sub_affix_list.push(RelicAffix {
                    affix_id: stat.sub_affix_id,
                    cnt: stat.count,
                    step: stat.step,
                })
            }
        }
    }

    // blessings
    for blessing in &player.battle_config.blessings {
        let mut buffs = BattleBuff {
            id: blessing.id,
            level: blessing.level,
            wave_flag: 0xffffffff,
            owner_id: 0xffffffff,
            ..Default::default()
        };

        if let Some(dynamic_value) = &blessing.dynamic_key {
            buffs
                .dynamic_values
                .insert(dynamic_value.key.clone(), dynamic_value.value as f32);
        };

        for dynamic_value in &blessing.dynamic_values {
            if buffs.dynamic_values.contains_key(&dynamic_value.key) {
                continue;
            };
            buffs
                .dynamic_values
                .insert(dynamic_value.key.clone(), dynamic_value.value as f32);
        }

        battle_info.buff_list.push(buffs);
    }

    // pf score object
    if player.battle_config.battle_type == BattleType::PF {
        battle_info.battle_target_info.insert(
            1,
            BattleTargetList {
                battle_target_list: vec![BattleTarget {
                    id: 10001,
                    progress: 0,
                    ..Default::default()
                }],
            },
        );

        for i in 2..=4 {
            battle_info
                .battle_target_info
                .insert(i, BattleTargetList::default());
        }

        battle_info.battle_target_info.insert(
            5,
            BattleTargetList {
                battle_target_list: vec![
                    BattleTarget {
                        id: 2001,
                        progress: 0,
                        ..Default::default()
                    },
                    BattleTarget {
                        id: 2002,
                        progress: 0,
                        ..Default::default()
                    },
                ],
            },
        );
    }

    // Apocalyptic Shadow
    if player.battle_config.battle_type == BattleType::AS {
        battle_info.battle_target_info.insert(
            1,
            BattleTargetList {
                battle_target_list: vec![BattleTarget {
                    id: 90005,
                    progress: 0,
                    ..Default::default()
                }],
            },
        );
    }

    //  SU
    if player.battle_config.battle_type == BattleType::SU {
        battle_info
            .event_battle_info_list
            .push(BattleEventBattleInfo {
                battle_event_id: player.battle_config.path_resonance_id,
                status: Some(BattleEventInitedData {
                    sp_bar: Some(SpBarInfo {
                        cur_sp: 10_000,
                        max_sp: 10_000,
                    }),
                }),
            })
    }

    // monsters
    battle_info.monster_wave_list = Monster::to_scene_monster_waves(&player.battle_config.monsters);

    battle_info
}
