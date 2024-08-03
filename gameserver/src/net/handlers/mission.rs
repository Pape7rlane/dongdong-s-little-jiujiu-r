use super::*;

pub async fn on_get_mission_status_cs_req(
    session: &mut PlayerSession,
    body: &GetMissionStatusCsReq,
) -> Result<()> {
    let rsp = GetMissionStatusScRsp {
        retcode: 0,
        finished_main_mission_id_list: body.main_mission_id_list.clone(),
        sub_mission_status_list: body
            .sub_mission_id_list
            .iter()
            .map(|id| Mission {
                id: *id,
                progress: 1,
                status: MissionStatus::MissionFinish.into(),
            })
            .collect(),
        ..Default::default()
    };

    session.send(CMD_GET_MISSION_STATUS_SC_RSP, rsp).await
}