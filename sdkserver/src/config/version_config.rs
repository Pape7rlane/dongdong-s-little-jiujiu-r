use std::{collections::HashMap, sync::OnceLock};

use serde::Deserialize;

const DEFAULT_VERSIONS: &str = include_str!("../../../versions.json");

#[derive(Deserialize)]
pub struct VersionConfig {
    pub asset_bundle_url: String,
    pub ex_resource_url: String,
    pub lua_url: String,
    // pub lua_version: String,
}

pub fn instance() -> &'static HashMap<String, VersionConfig> {
    static INSTANCE: OnceLock<HashMap<String, VersionConfig>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        const CONFIG_PATH: &str = "versions.json";

        let data = std::fs::read_to_string(CONFIG_PATH).unwrap_or_else(|_| {
            std::fs::write(CONFIG_PATH, DEFAULT_VERSIONS).expect("Failed to create versions file");
            DEFAULT_VERSIONS.to_string()
        });

        serde_json::from_str(&data).unwrap_or_else(|e| panic!("Failed to parse versions.json: {e}"))
    })
}
