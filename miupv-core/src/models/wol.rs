//! Wake-on-LAN (WOL) remote machine management models.

use super::utils::deserialize_flexible_bool;
use serde::{Deserialize, Serialize};

/// Remote computer/workstation in a university laboratory or office.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Equipo {
    /// Equipment identifier (e.g. "LAB-01").
    #[serde(alias = "id", alias = "ID", alias = "cod_equipo")]
    pub id: String,

    /// Machine hostname or human-readable description.
    #[serde(alias = "nombre", alias = "NOMBRE", alias = "nom_equipo")]
    pub nombre: String,

    /// Current reported status (e.g. "OFF", "ON", "UNKNOWN").
    #[serde(alias = "estado", alias = "ESTADO", default)]
    pub estado: Option<String>,

    /// Network IPv4 address.
    #[serde(alias = "ip", alias = "IP", default)]
    pub ip: Option<String>,

    /// Hardware MAC address.
    #[serde(alias = "mac", alias = "MAC", default)]
    pub mac: Option<String>,

    /// Campus location (e.g. "Vera", "Gandia", "Alcoi").
    #[serde(alias = "campus", alias = "CAMPUS", default)]
    pub campus: Option<String>,
}

/// Result returned from triggering a Wake-on-LAN magic packet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WakeupResult {
    /// True if the wake command was successfully queued or dispatched.
    #[serde(default, deserialize_with = "deserialize_flexible_bool")]
    pub success: bool,

    /// Informational message from the wake server.
    #[serde(alias = "mensaje", alias = "MENSAJE", alias = "msg", default)]
    pub mensaje: Option<String>,
}
