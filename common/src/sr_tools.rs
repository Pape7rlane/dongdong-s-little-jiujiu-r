use proto::{Avatar, AvatarSkillTree, MultiPathAvatarTypeInfo};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::structs::{
    // AllowedLanguages,
    avatar::{AvatarJson, MultiPathAvatar},
    battle::BattleConfig,
    lightcone::Lightcone,
    persistent::Persistent,
    relic::Relic,
    scene::{Position, Scene},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FreesrData {
    pub lightcones: Vec<Lightcone>,
    pub relics: Vec<Relic>,
    pub avatars: HashMap<u32, AvatarJson>,
    #[serde(default)]
    pub battle_config: BattleConfig,

    // Non freesr-data.json fields
    #[serde(skip_serializing, skip_deserializing)]
    pub lineups: BTreeMap<u32, u32>,
    #[serde(skip_serializing, skip_deserializing)]
    pub position: Position,
    #[serde(skip_serializing, skip_deserializing)]
    pub scene: Scene,
    #[serde(skip_serializing, skip_deserializing)]
    pub main_character: MultiPathAvatar,
    #[serde(skip_serializing, skip_deserializing)]
    pub march_type: MultiPathAvatar,
    // #[serde(skip_serializing, skip_deserializing)]
    // pub game_language: AllowedLanguages,
    // #[serde(skip_serializing, skip_deserializing)]
    // pub voice_langauge: AllowedLanguages,
}

impl FreesrData {
    pub fn get_avatar_proto(&self, avatar_id: u32) -> Option<Avatar> {
        let avatar = self.avatars.get(&avatar_id)?;
        let lightcone = self.lightcones.iter().find(|l| l.equip_avatar == avatar_id);
        let relics = self.relics.iter().filter(|r| r.equip_avatar == avatar_id);

        // TODO: HARDCODED
        let base_avatar_id = if avatar.avatar_id > 8000 {
            8001
        } else if avatar.avatar_id == 1001 || avatar.avatar_id == 1224 {
            1001
        } else {
            avatar.avatar_id
        };

        Some(Avatar {
            base_avatar_id,
            level: avatar.level,
            promotion: avatar.promotion,
            rank: avatar.data.rank,
            skilltree_list: avatar
                .data
                .skills
                .iter()
                .map(|v| AvatarSkillTree {
                    point_id: *v.0,
                    level: *v.1,
                })
                .collect::<Vec<_>>(),
            equipment_unique_id: lightcone.map(|v| v.get_unique_id()).unwrap_or_default(),
            first_met_timestamp: 1712924677,
            equip_relic_list: relics.map(|v| v.into()).collect::<Vec<_>>(),
            ..Default::default()
        })
    }

    pub fn get_avatar_multipath_proto(&self, avatar_id: u32) -> Option<MultiPathAvatarTypeInfo> {
        let avatar = self.avatars.get(&avatar_id)?;
        let mp_type = MultiPathAvatar::from(avatar_id);

        if mp_type == MultiPathAvatar::Unk {
            return None;
        }

        Some(MultiPathAvatarTypeInfo {
            avatar_id: mp_type as i32,
            rank: avatar.data.rank,
            equip_relic_list: self
                .relics
                .iter()
                .filter(|relic| relic.equip_avatar == mp_type as u32)
                .map(|relic| relic.into())
                .collect(),
            skilltree_list: avatar
                .data
                .skills
                .iter()
                .map(|(point_id, level)| AvatarSkillTree {
                    point_id: *point_id,
                    level: *level,
                })
                .collect(),
            path_equipment_id: self
                .lightcones
                .iter()
                .find(|v| v.equip_avatar == mp_type as u32)
                .map(|v| v.get_unique_id())
                .unwrap_or_default(),
            dressed_skin_id: 0,
            ..Default::default()
        })
    }

    pub fn get_multi_path_info(&self) -> Vec<MultiPathAvatarTypeInfo> {
        MultiPathAvatar::to_vec()
            .into_iter()
            .filter_map(|mp_type| {
                if mp_type.is_mc() && mp_type.get_gender() != self.main_character.get_gender() {
                    return Option::None;
                }

                let avatar_info = self.avatars.get(&((mp_type) as u32))?;
                Some(MultiPathAvatarTypeInfo {
                    avatar_id: mp_type as i32,
                    rank: avatar_info.data.rank,
                    equip_relic_list: self
                        .relics
                        .iter()
                        .filter(|relic| relic.equip_avatar == mp_type as u32)
                        .map(|relic| relic.into())
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
                    path_equipment_id: self
                        .lightcones
                        .iter()
                        .find(|v| v.equip_avatar == mp_type as u32)
                        .map(|v| v.get_unique_id())
                        .unwrap_or_default(),
                    dressed_skin_id: 0,
                    ..Default::default()
                })
            })
            .collect()
    }
}

impl FreesrData {
    pub async fn load() -> Self {
        let mut freesr_data: FreesrData = tokio::fs::read_to_string("freesr-data.json")
            .await
            .map(|v| {
                serde_json::from_str::<FreesrData>(&v)
                    .expect("freesr-data.json is broken, pls redownload")
            })
            .expect("failed to read freesr-data.json");

        let persistent: Persistent = serde_json::from_str(
            &tokio::fs::read_to_string("persistent")
                .await
                .unwrap_or_default(),
        )
        .unwrap_or_default();

        freesr_data.lineups = persistent.lineups;
        freesr_data.position = persistent.position;
        freesr_data.scene = persistent.scene;
        freesr_data.main_character = persistent.main_character;
        freesr_data.march_type = persistent.march_type;
        // freesr_data.game_language = persistent.game_language;
        // freesr_data.voice_langauge = persistent.voice_language;

        // remove unequipped lightcones
        if freesr_data.lightcones.len() > 1500 {
            freesr_data.lightcones.retain(|v| v.equip_avatar != 0);
        }

        // remove unequipped relics
        if freesr_data.relics.len() > 2000 {
            freesr_data.relics.retain(|v| v.equip_avatar != 0);
        }

        freesr_data.verify_lineup().await;

        if freesr_data.march_type as u32 > 8000 {
            freesr_data.march_type = MultiPathAvatar::MarchHunt;
        }

        freesr_data
    }

    pub async fn update(&mut self) {
        *self = Self::load().await
    }

    async fn verify_lineup(&mut self) {
        if self.lineups.is_empty() {
            self.lineups = BTreeMap::<u32, u32>::from([(0, 8001), (1, 0), (2, 0), (3, 0)])
        } else if self.lineups.len() < 4 {
            for i in self.lineups.len()..4 {
                self.lineups.insert(i as u32, 0);
            }
        }
        self.save_persistent().await;
    }

    pub async fn save_persistent(&self) {
        let _ = tokio::fs::write(
            "persistent",
            serde_json::to_string_pretty(&Persistent {
                lineups: self.lineups.clone(),
                main_character: self.main_character,
                position: self.position.clone(),
                scene: self.scene.clone(),
                march_type: self.march_type,
                // game_language: self.game_language,
                // voice_language: self.voice_langauge,
            })
            .unwrap(),
        )
        .await;
        tracing::info!("persistent saved");
    }
}
