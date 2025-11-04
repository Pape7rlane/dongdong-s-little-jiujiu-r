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
    res.server_time_ms = body.client_time_ms;
    res.download_data = Some(ClientDownloadData {
        version: 51,
        time: res.server_time_ms as i64,
        data: rbase64::decode("bG9jYWwgTUFYX0ZQUyA9IDM2MApsb2NhbCBCRVRBX1dNID0gIiIKCmxvY2FsIGZ1bmN0aW9uIGxvZyhtc2cpCmVuZAoKbG9jYWwgZnVuY3Rpb24gcGF0Y2hGcHMoKQogICAgQ1MuVW5pdHlFbmdpbmUuQXBwbGljYXRpb24udGFyZ2V0RnJhbWVSYXRlID0gTUFYX0ZQUwogICAgQ1MuVW5pdHlFbmdpbmUuUXVhbGl0eVNldHRpbmdzLnZTeW5jQ291bnQgPSAwCmVuZAoKbG9jYWwgZnVuY3Rpb24gcGF0Y2hCZXRhV20oKQogICAgLS0gQmV0YSBXYXRlcm1hcmsKICAgIENTLlVuaXR5RW5naW5lLkdhbWVPYmplY3QuRmluZCgiVUlSb290L0Fib3ZlRGlhbG9nL0JldGFIaW50RGlhbG9nKENsb25lKSIpOkdldENvbXBvbmVudEluQ2hpbGRyZW4odHlwZW9mKENTLlJQRy5DbGllbnQuTG9jYWxpemVkVGV4dCkpLnRleHQgPSBCRVRBX1dNCgogICAgLS0gVUlEIFdhdGVybWFyawogICAgQ1MuVW5pdHlFbmdpbmUuR2FtZU9iamVjdC5GaW5kKCJWZXJzaW9uVGV4dCIpOkdldENvbXBvbmVudEluQ2hpbGRyZW4odHlwZW9mKENTLlJQRy5DbGllbnQuTG9jYWxpemVkVGV4dCkpLnRleHQgPQogICAgICAgICI8Y29sb3I9IzAwZTFmZj5Sb2JpblNSISB8IDwvY29sb3I+IiAuLgogICAgICAgICI8Y29sb3I9IzAwZTFmZj4iIC4uCiAgICAgICAgQ1MuUlBHLkNsaWVudC5HbG9iYWxWYXJzLnNfVmVyc2lvbkRhdGE6R2V0U2VydmVyUGFrVHlwZVZlcnNpb24oKSAuLgogICAgICAgICI8L2NvbG9yPiIKZW5kCgoKbG9jYWwgZnVuY3Rpb24gbWFpbigpCiAgICBwYXRjaEJldGFXbSgpCmVuZAoKeHBjYWxsKG1haW4sIGxvZyk=").unwrap(),
        ..Default::default()
    });
}

pub async fn on_player_login_finish_cs_req(
    session: &mut PlayerSession,
    _req: &PlayerLoginFinishCsReq,
    _res: &mut PlayerLoginFinishScRsp,
) -> Result<()> {
    session
        .send(ContentPackageSyncDataScNotify {
            data: Some(ContentPackageData {
                content_package_list: [
                    200001, 200002, 200003, 200004, 200005, 200006, 200007, 200008, 150017, 150015,
                    150021, 150018, 130011, 130012, 130013, 150025, 140006, 150026, 130014, 150034,
                    150029, 150035, 150041, 150039, 150045,
                ]
                .into_iter()
                .map(|v| ContentPackageInfo {
                    status: ContentPackageStatus::Finished.into(),
                    content_id: v,
                })
                .collect(),
                ..Default::default()
            }),
        })
        .await?;

    Ok(())
}
