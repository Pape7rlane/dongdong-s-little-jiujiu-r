use scene_entity_info::Entity;
use scene_entity_refresh_info::RefreshType;

use crate::net::tools::{self, AvatarJson, FreesrData};

use super::*;

pub async fn on_get_all_lineup_data_cs_req(
    session: &mut PlayerSession,
    _body: &GetAllLineupDataCsReq,
) -> Result<()> {
    let player = tools::FreesrData::load().await;
    let lineup = LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        mp: 5,
        max_mp: 5,
        avatar_list: AvatarJson::to_lineup_avatars(&player),
        ..Default::default()
    };

    session
        .send(
            CMD_GET_ALL_LINEUP_DATA_SC_RSP,
            GetAllLineupDataScRsp {
                lineup_list: vec![lineup],
                ..Default::default()
            },
        )
        .await
}

pub async fn on_get_cur_lineup_data_cs_req(
    session: &mut PlayerSession,
    _body: &GetCurLineupDataCsReq,
) -> Result<()> {
    let player = tools::FreesrData::load().await;
    let lineup = LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        mp: 5,
        max_mp: 5,
        avatar_list: AvatarJson::to_lineup_avatars(&player),
        is_virtual: false,
        plane_id: 0,
        ..Default::default()
    };

    session
        .send(
            CMD_GET_CUR_LINEUP_DATA_SC_RSP,
            GetCurLineupDataScRsp {
                lineup: Some(lineup),
                ..Default::default()
            },
        )
        .await
}

pub async fn on_join_lineup_cs_req(
    session: &mut PlayerSession,
    body: &JoinLineupCsReq,
) -> Result<()> {
    // update lineups
    // TODO: FIX THESE SHIT
    {
        let mut player = tools::FreesrData::load().await;
        let lineups = &mut player.lineups;
        lineups.insert(
            body.slot,
            if body.base_avatar_id == 8001 {
                player.main_character as u32
            } else if body.base_avatar_id == 1001 {
                player.march_type as u32
            } else {
                body.base_avatar_id
            },
        );
        player.save_lineup().await;
    }

    {
        let player = tools::FreesrData::load().await;

        refresh_lineup(session, &player).await?;
    }

    session
        .send(CMD_JOIN_LINEUP_SC_RSP, JoinLineupScRsp::default())
        .await
}

pub async fn on_replace_lineup_cs_req(
    _session: &mut PlayerSession,
    req: &ReplaceLineupCsReq,
) -> Result<()> {
    {
        let mut player = tools::FreesrData::load().await;

        let lineups = &mut player.lineups;
        for (slot, avatar_id) in &mut *lineups {
            if let Some(lineup) = req.slots.get(*slot as usize) {
                *avatar_id = if lineup.id == 8001 {
                    player.main_character as u32
                } else if lineup.id == 1001 {
                    player.march_type as u32
                } else {
                    lineup.id
                };
            } else {
                *avatar_id = 0;
            }
        }
        player.save_lineup().await;
    }

    {
        let player = tools::FreesrData::load().await;

        refresh_lineup(_session, &player).await?;
    }

    _session
        .send(CMD_JOIN_LINEUP_SC_RSP, JoinLineupScRsp::default())
        .await
}

pub async fn on_quit_lineup_cs_req(
    _session: &mut PlayerSession,
    _: &QuitLineupCsReq,
) -> Result<()> {
    _session
        .send(CMD_JOIN_LINEUP_SC_RSP, JoinLineupScRsp::default())
        .await
}

async fn refresh_lineup(sess: &mut PlayerSession, player: &FreesrData) -> Result<()> {
    let lineup = LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        avatar_list: AvatarJson::to_lineup_avatars(player),
        max_mp: 5,
        mp: 5,
        ..Default::default()
    };

    sess.send(
        CMD_SCENE_GROUP_REFRESH_SC_NOTIFY,
        SceneGroupRefreshScNotify {
            group_refresh_info: vec![SceneGroupRefreshInfo {
                group_id: 0,
                state: 0,
                group_refresh_type: 0,
                refresh_entity: player
                    .lineups
                    .iter()
                    .map(|(idx, v)| SceneEntityRefreshInfo {
                        refresh_type: Some(RefreshType::AddEntity(SceneEntityInfo {
                            entity: Some(Entity::Actor(SceneActorInfo {
                                avatar_type: AvatarType::AvatarFormalType.into(),
                                base_avatar_id: *v,
                                map_layer: 0,
                                uid: 25,
                            })),
                            entity_id: idx + 1,
                            group_id: 0,
                            inst_id: 0,
                            ..Default::default()
                        })),
                    })
                    .collect(),
            }],
        },
    )
    .await?;

    sess.send(
        CMD_SYNC_LINEUP_NOTIFY,
        SyncLineupNotify {
            lineup: Some(lineup),
            reason_list: vec![],
        },
    )
    .await
}

pub async fn on_change_lineup_leader_cs_req(
    session: &mut PlayerSession,
    body: &ChangeLineupLeaderCsReq,
) -> Result<()> {
    session
        .send(
            CMD_CHANGE_LINEUP_LEADER_SC_RSP,
            ChangeLineupLeaderScRsp {
                slot: body.slot,
                retcode: 0,
            },
        )
        .await
}
