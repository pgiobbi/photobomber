use serde::{Deserialize, Serialize};

/// Struct for stages retrieved from the Tomorrowland API.
#[derive(Serialize, Deserialize)]
pub struct TomorrowlandStage {
    /// Stage id.
    pub id: String,
    /// Stage name.
    pub name: String,
}

/// Internal stages
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
    // TODO: add other areas, such as Dreamville etc.
}
