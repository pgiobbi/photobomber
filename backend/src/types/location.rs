use crate::types::state::LocationState;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLocationResponse {
    pub location: Option<Stage>,
    pub updated_at: Option<i64>,
    pub updated_ago: Option<i64>,
}

impl From<&LocationState> for GetLocationResponse {
    fn from(location_state: &LocationState) -> Self {
        let updated_ago = location_state
            .updated_at
            .map(|updated_at| Utc::now().timestamp_millis() - updated_at);
        GetLocationResponse {
            location: location_state.location.clone(),
            updated_at: location_state.updated_at,
            updated_ago,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Stage {
    #[serde(rename = "Main Stage")]
    MainStage,
    #[serde(rename = "Freedom Stage")]
    FreedomStage,
    #[serde(rename = "The Rose Garden")]
    RoseGarden,
    #[serde(rename = "Elixir")]
    Elixir,
    #[serde(rename = "Cage")]
    Cage,
    #[serde(rename = "The Rave Cave")]
    RaveCave,
    #[serde(rename = "Planaxis")]
    Planaxis,
    #[serde(rename = "Rise By Coke Studio")]
    Rise,
    #[serde(rename = "Atmosphere")]
    Atmosphere,
    #[serde(rename = "CORE")]
    Core,
    #[serde(rename = "Crystal Garden")]
    CrystalGarden,
    #[serde(rename = "The Library")]
    Library,
    #[serde(rename = "Melodia")]
    Melodia,
    #[serde(rename = "House Of Fortune")]
    HouseOfFortune,
    #[serde(rename = "Roaming Around...")]
    RoamingAround,

    // ?? below ??
    #[serde(rename = "Portal to Paradise by Tulum")]
    PortalToParadise,
    #[serde(rename = "Sylvira")]
    Sylvira,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostLocationRequest {
    pub location: Option<Stage>,
}
