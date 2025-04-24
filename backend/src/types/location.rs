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
        let updated_ago = match location_state.updated_at {
            None => None,
            Some(updated_at) => Some(Utc::now().timestamp_millis() - updated_at),
        };
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
    #[serde(rename = "Other")]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostLocationRequest {
    pub location: Option<Stage>,
}
