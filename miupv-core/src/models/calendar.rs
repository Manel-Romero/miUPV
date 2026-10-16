//! Academic calendar event models.

use super::utils::deserialize_flexible_bool;
use serde::{Deserialize, Serialize};

/// An academic or personal calendar event from UPV iCal service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalEvent {
    /// Unique event identifier.
    #[serde(
        alias = "id",
        alias = "ID",
        alias = "id_evento",
        alias = "idEvento",
        alias = "IDEVENTO"
    )]
    pub id: String,

    /// Event title (e.g. subject name or class topic).
    #[serde(
        alias = "titulo",
        alias = "TITULO",
        alias = "title",
        alias = "nom_evento"
    )]
    pub titulo: String,

    /// Detailed description or location notes.
    #[serde(
        alias = "descripcion",
        alias = "DESCRIPCION",
        alias = "description",
        alias = "desc",
        alias = "DESC",
        default
    )]
    pub descripcion: Option<String>,

    /// Start timestamp (e.g. "2026-09-16T08:30:00.000Z").
    #[serde(
        alias = "fecha_ini",
        alias = "FECHA_INI",
        alias = "fechaIni",
        alias = "FECHAINI",
        alias = "start_date"
    )]
    pub fecha_ini: String,

    /// End timestamp.
    #[serde(
        alias = "fecha_fin",
        alias = "FECHA_FIN",
        alias = "fechaFin",
        alias = "FECHAFIN",
        alias = "end_date"
    )]
    pub fecha_fin: String,

    /// Event type (e.g. "Clases", "Exámenes", "TEORIA").
    #[serde(
        alias = "tipo",
        alias = "TIPO",
        alias = "type",
        alias = "categoria",
        alias = "CATEGORIA",
        default
    )]
    pub tipo: Option<String>,

    /// Event location (e.g. "AULA 1E 1.3 (Edificio 1E)").
    #[serde(alias = "lugar", alias = "LUGAR", alias = "location", default)]
    pub lugar: Option<String>,

    /// Color hex code or label for display.
    #[serde(
        alias = "color",
        alias = "COLOR",
        alias = "categoriaDeco",
        alias = "CATEGORIADECO",
        default
    )]
    pub color: Option<String>,

    /// True if the event spans the entire day.
    #[serde(
        alias = "todo_dia",
        alias = "TODO_DIA",
        alias = "all_day",
        alias = "diaCompleto",
        alias = "DIACOMPLETO",
        default,
        deserialize_with = "deserialize_flexible_bool"
    )]
    pub todo_dia: bool,
}
