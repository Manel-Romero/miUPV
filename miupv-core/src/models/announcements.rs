//! Institutional announcements and notifications (Avisos).

use super::utils::deserialize_flexible_bool;
use serde::{Deserialize, Serialize};

/// Official announcement or notice from a university faculty, school, or department.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Aviso {
    /// Unique announcement identifier.
    #[serde(alias = "id", alias = "ID", alias = "id_aviso")]
    pub id: String,

    /// Title or subject of the notice.
    #[serde(alias = "titulo", alias = "TITULO", alias = "asunto")]
    pub titulo: String,

    /// Content or body text.
    #[serde(alias = "texto", alias = "TEXTO", alias = "cuerpo", default)]
    pub texto: Option<String>,

    /// Publication date.
    #[serde(alias = "fecha", alias = "FECHA", default)]
    pub fecha: Option<String>,

    /// Issuing university center (e.g. "ETSINF", "ETSII").
    #[serde(alias = "centro", alias = "CENTRO", alias = "nom_centro", default)]
    pub centro: Option<String>,

    /// High priority / urgent announcement indicator.
    #[serde(
        alias = "importante",
        alias = "IMPORTANTE",
        default,
        deserialize_with = "deserialize_flexible_bool"
    )]
    pub importante: bool,
}
