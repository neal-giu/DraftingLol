use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRole {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTeamMember {
    pub id: String,
    pub handle: String,
    pub role: UiRole,
    pub comfort_picks: Vec<String>,
    pub focus: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub team_name: String,
    pub patch: String,
    pub preferred_style: String,
    pub target_identity: Vec<crate::domain::composition::CompositionIdentity>,
    pub members: Vec<UiTeamMember>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveConfigPayload {
    pub config: AppConfig,
}

fn config_path() -> PathBuf {
    std::env::temp_dir().join("draftinglol_desktop_config.json")
}

fn default_config() -> AppConfig {
    AppConfig {
        team_name: "Drafting Laboratory".into(),
        patch: "15.6".into(),
        preferred_style: "Front-to-back propre avec enclenchement fort".into(),
        target_identity: vec![
            crate::domain::composition::CompositionIdentity::Engage,
            crate::domain::composition::CompositionIdentity::FrontToBack,
            crate::domain::composition::CompositionIdentity::ProtectCarry,
        ],
        members: vec![
            UiTeamMember {
                id: "top-alto".into(),
                handle: "Alto".into(),
                role: UiRole::Top,
                comfort_picks: vec!["Ornn".into(), "Jax".into(), "Ksante".into()],
                focus: "frontline stable".into(),
            },
            UiTeamMember {
                id: "jungle-mira".into(),
                handle: "Mira".into(),
                role: UiRole::Jungle,
                comfort_picks: vec!["Sejuani".into(), "Vi".into(), "Poppy".into()],
                focus: "setup engage".into(),
            },
            UiTeamMember {
                id: "mid-zen".into(),
                handle: "Zen".into(),
                role: UiRole::Mid,
                comfort_picks: vec!["Ahri".into(), "Orianna".into(), "Taliyah".into()],
                focus: "prio mid + picks".into(),
            },
            UiTeamMember {
                id: "adc-lyra".into(),
                handle: "Lyra".into(),
                role: UiRole::Bottom,
                comfort_picks: vec!["Smolder".into(), "Jinx".into(), "Zeri".into()],
                focus: "late game carry".into(),
            },
            UiTeamMember {
                id: "sup-kai".into(),
                handle: "Kai".into(),
                role: UiRole::Support,
                comfort_picks: vec!["Nautilus".into(), "Renata".into(), "Rell".into()],
                focus: "vision control".into(),
            },
        ],
    }
}

#[tauri::command]
pub fn load_app_config() -> Result<AppConfig, String> {
    let path = config_path();

    if !path.exists() {
        return Ok(default_config());
    }

    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_app_config(payload: SaveConfigPayload) -> Result<AppConfig, String> {
    let path = config_path();
    let body = serde_json::to_string_pretty(&payload.config).map_err(|error| error.to_string())?;

    fs::write(path, body).map_err(|error| error.to_string())?;

    Ok(payload.config)
}
