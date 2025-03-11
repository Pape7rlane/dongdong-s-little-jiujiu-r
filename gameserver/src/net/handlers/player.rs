use std::collections::HashMap;

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
        data: rbase64::decode("BAEwB0dldFR5cGUIR2V0RmllbGQIR2V0VmFsdWUCQ1MGU3lzdGVtC0NvbGxlY3Rpb25zB0dlbmVyaWMETGlzdAZTdHJpbmcCY24DQWRkAmVuAmtyAmpwB1RvQXJyYXkDUlBHBkNsaWVudApHbG9iYWxWYXJzFXNfTG9jYWxpemF0aW9uTWFuYWdlchRfYWxsb3dlZExhbmd1YWdlS2V5cxFfYWxsb3dlZEF1ZGlvS2V5cwhHYW1lQ29yZRlBbGxvd2VkTGFuZ3VhZ2VFeGNlbFRhYmxlCGRhdGFEaWN0BXBhaXJzDExhbmd1YWdlTGlzdAtVbml0eUVuZ2luZQtBcHBsaWNhdGlvbgdNQVhfRlBTD3RhcmdldEZyYW1lUmF0ZQ9RdWFsaXR5U2V0dGluZ3MKdlN5bmNDb3VudApHYW1lT2JqZWN0BEZpbmQoVUlSb290L0Fib3ZlRGlhbG9nL0JldGFIaW50RGlhbG9nKENsb25lKQ1Mb2NhbGl6ZWRUZXh0BnR5cGVvZhZHZXRDb21wb25lbnRJbkNoaWxkcmVuB0JFVEFfV00EdGV4dAtWZXJzaW9uVGV4dCI8Y29sb3I9IzAwZTFmZj5Sb2JpblNSISB8IDwvY29sb3I+Dzxjb2xvcj0jMDBlMWZmPg1zX1ZlcnNpb25EYXRhF0dldFNlcnZlclBha1R5cGVWZXJzaW9uCDwvY29sb3I+BnhwY2FsbApJU19QQVRDSEVECQEBAAAAAAEWAAEAAAACAAAABgIAAAAADRQCABsAAAAAFQICAgYEAQAEBT4AFAICkQEAAAAVAgQCBgQAABQCAvMCAAAAFQIDABYCAAADAwEDAgMDAAQAAAAFAAAAAAAfDAIDAAIEAMAPAQL5BAAAAA8AAesFAAAADAEHAAYEAMAVAAICBgEAABUBAQIFBAgAFAIBXwkAAAAVAgMBBQQKABQCAV8JAAAAFQIDAQUECwAUAgFfCQAAABUCAwEFBAwAFAIBXwkAAAAVAgMBFAIBOQ0AAAAVAgIAFgIAAA4DBAMFAwYEAgQAwAMHAwgDCQQGBADAAwoDCwMMAw0DDgMPAAsAAAAGAAAAAAA/DAIDAAIEAMAPAQKsBAAAAA8AAaAFAAAAFAEAGwYAAAAVAQICBQMHAAQEPgAUAQGRCAAAABUBBAIGAwAAFAEB8wkAAAAVAQMCBQQKABQCAV8LAAAAFQIDAQUEDAAUAgFfCwAAABUCAwEFBA0AFAIBXwsAAAAVAgMBBQQOABQCAV8LAAAAFQIDARQCABsGAAAAFQICAgUEDwAEBT4AFAICkQgAAAAVAgQCBgQAABQCAvMJAAAAFQIDAgUFCgAUAwJfCwAAABUDAwEFBQwAFAMCXwsAAAAVAwMBBQUNABQDAl8LAAAAFQMDAQUFDgAUAwJfCwAAABUDAwEWAAEAEAMEAxADEQQCBADAAxIDEwMBAxQDAgMDAwoDCwMMAw0DDgMVABQAAAAHAAEAAAASDAIDAAIEAMAPAQLwBAAAAA8AAbkFAAAACQEAABUBAQIMAgcAAABgQAYDAAAVAgIEQQICABABBjIIAAAAPgL9/wIAAAAWAAEACQMEAxADFgQCBADAAxcDGAMZBAAAYEADGgAiAAAAAgAAAAAADAwAAwACBADABwEAygQAAAAQAQADBQAAAAwABwAGBADABAEAABABANAIAAAAFgABAAkDBAMbAxwEAgQAwAMdAx4DHwQGBADAAyAALAAAAAkAAAAAADcMAQMAAgQAwA8AAXYEAAAABQEFABUAAgIMBAgABxgAwA8DBBQJAAAATSwDAgwCCwAAAKBAFQICAhQAALEMAAAAFQADAgwBDgAAANBAEAEA1Q8AAAAMAQMAAgQAwA8AAXYEAAAABQEQABUAAgIMBAgABxgAwA8DBBQJAAAATSwDAgwCCwAAAKBAFQICAhQAALEMAAAAFQADAgUCEQAFAxIADAgIAAcYAMAPBwisEwAAAA8GByMUAAAAFAYGkRUAAAAVBgICBgQGAAUFFgA1AQIFEAEA1Q8AAAAWAAEAFwMEAxsDIQQCBADAAyIDIwMQAxEEBxgAwAMkAyUEAACgQAMmAycEAADQQAMoAykDKgMrAxIDLAMtAy4AMQAAAAMABQAAACQHAADKAAAAAFEABAAAAACABABoAQgAAMoAAAAADAACAAAAEEAJAQAACQIBABUAAwEMAAIAAAAQQAkBAgAJAgEAFQADAQcAACsDAAAAUgACAAEAAIAWAAEAAwABAAgAACsDAAAADAACAAAAEEAJAQMACQIBABUAAwEMAAIAAAAQQAkBBAAJAgEAFQADARYAAQAEAx0DLwQAABBAAzAAPwAAAAsAAAEAABVFAAAARAAAAEQBAQBEAgIARAMDAEQEBABKAAIARAUFAEQGBgBEBwcASgAFAEoAAABKAAYASgAEAEoAAwAMCAkAAACAQAYJBwAGCgAAFQgDARYAAQAKBgAGAQYCBgMGBAYFBgYGBwMvBAAAgEAIAAECAwQFBgcBAAAACA==").unwrap(),
        haehhcpoapp: 0
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
    session: &mut PlayerSession,
    _req: &GetMultiPathAvatarInfoCsReq,
    res: &mut GetMultiPathAvatarInfoScRsp,
) {
    let Some(json) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    res.current_multi_path_avatar_id = HashMap::from([
        (8001, json.main_character.get_type().into()),
        (1001, json.march_type.get_type().into()),
    ]);

    res.multi_path_avatar_type_info_list = json.get_multi_path_info();
}
