//! Library study cabin models and reservations.

use super::utils::deserialize_flexible_bool;
use serde::{Deserialize, Serialize};

/// Study cabin available time slot or resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cabina {
    /// Slot identifier (e.g. "12345").
    #[serde(alias = "id", alias = "idFranja", alias = "id_franja")]
    pub id: String,

    /// Cabin name or code (e.g. "Cabina 1 - Central").
    #[serde(alias = "nombre", alias = "recurso", alias = "espacio", alias = "nom_recurso")]
    pub nombre: String,

    /// Library campus or building name.
    #[serde(alias = "biblioteca", alias = "entidad", alias = "nom_entidad", default)]
    pub biblioteca: Option<String>,

    /// Date string (e.g. "2025-03-15").
    #[serde(alias = "fecha", default)]
    pub fecha: Option<String>,

    /// Start time string (e.g. "10:00").
    #[serde(alias = "hora_ini", alias = "horaInicio", alias = "hora_inicio", default)]
    pub hora_ini: Option<String>,

    /// End time string (e.g. "12:00").
    #[serde(alias = "hora_fin", alias = "horaFin", alias = "hora_final", default)]
    pub hora_fin: Option<String>,

    /// Whether this cabin slot is available for booking.
    #[serde(alias = "libre", default, deserialize_with = "deserialize_flexible_bool")]
    pub libre: bool,

    /// Maximum capacity of the cabin.
    #[serde(alias = "max_personas", alias = "aforo", alias = "capacidad", default)]
    pub max_personas: Option<u32>,
}

/// A confirmed reservation for a study cabin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reserva {
    /// Reservation ID.
    #[serde(alias = "id", alias = "idReserva", alias = "id_reserva")]
    pub id: String,

    /// Associated cabin time slot ID.
    #[serde(alias = "cabina_id", alias = "idFranja", alias = "franjaId", default)]
    pub cabina_id: Option<String>,

    /// Resource / cabin name.
    #[serde(alias = "nombre", alias = "recurso", alias = "nom_recurso", default)]
    pub nombre: Option<String>,

    /// Library name.
    #[serde(alias = "biblioteca", alias = "entidad", default)]
    pub biblioteca: Option<String>,

    /// Booking date.
    #[serde(alias = "fecha", default)]
    pub fecha: Option<String>,

    /// Start time.
    #[serde(alias = "hora_ini", alias = "horaInicio", default)]
    pub hora_ini: Option<String>,

    /// End time.
    #[serde(alias = "hora_fin", alias = "horaFin", default)]
    pub hora_fin: Option<String>,
}
