//! Sports facility booking models.

use serde::{Deserialize, Serialize};

/// Sports court or activity reservation item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReservaDeporte {
    /// Reservation ID.
    #[serde(alias = "id", alias = "ID", alias = "id_reserva", default)]
    pub id: Option<String>,

    /// Campus (e.g. "Vera", "Gandia", "Alcoi").
    #[serde(alias = "campus", alias = "CAMPUS", default)]
    pub campus: Option<String>,

    /// Sport name (e.g. "padel", "tenis", "musculacion").
    #[serde(alias = "deporte", alias = "DEPORTE", alias = "nom_deporte", default)]
    pub deporte: Option<String>,

    /// Court / facility name.
    #[serde(alias = "pista", alias = "PISTA", alias = "nom_pista", default)]
    pub pista: Option<String>,

    /// Reservation date (e.g. "2025-03-15").
    #[serde(alias = "fecha", alias = "FECHA", default)]
    pub fecha: Option<String>,

    /// Start time (e.g. "17:00").
    #[serde(alias = "hora_ini", alias = "HORA_INI", alias = "horaInicio", default)]
    pub hora_ini: Option<String>,

    /// End time (e.g. "18:30").
    #[serde(alias = "hora_fin", alias = "HORA_FIN", alias = "horaFin", default)]
    pub hora_fin: Option<String>,

    /// Booking status (e.g. "CONFIRMADA", "PENDIENTE", "CANCELADA").
    #[serde(alias = "estado", alias = "ESTADO", default)]
    pub estado: Option<String>,
}

/// Sport category item from lista_deportes (e.g. Tenis, Baloncesto, Pádel).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeporteItem {
    /// Sport ID (e.g. "14" for BALONCESTO, "2" for TENIS).
    #[serde(alias = "deporte", alias = "DEPORTE", alias = "id")]
    pub id: String,

    /// Sport display name (e.g. "BALONCESTO", "TENIS").
    #[serde(alias = "nomdep", alias = "NOMDEP", alias = "nombre", alias = "name")]
    pub nombre: String,
}

/// Sports court containing available reservation slots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PistaConReservas {
    /// Court ID (e.g. "62").
    #[serde(alias = "pista", alias = "PISTA")]
    pub pista: String,

    /// Court name (e.g. "BALONCESTO EXTERIOR").
    #[serde(alias = "nompista", alias = "NOMPISTA")]
    pub nompista: String,

    /// List of reservation slots for this court.
    #[serde(alias = "reservas", alias = "RESERVAS", default)]
    pub reservas: Vec<SlotReserva>,
}

/// Individual time slot for a sports court.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlotReserva {
    /// Start time (e.g. "11:30").
    #[serde(alias = "hora_ini", alias = "HORA_INI")]
    pub hora_ini: String,

    /// End time (e.g. "12:30").
    #[serde(alias = "hora_fin", alias = "HORA_FIN")]
    pub hora_fin: String,

    /// Status (e.g. "L" for Libre / Available).
    #[serde(alias = "estado", alias = "ESTADO", default)]
    pub estado: Option<String>,

    /// Is a paid court (S/N).
    #[serde(alias = "espistapago", alias = "ESPISTAPAGO", default)]
    pub es_pago: Option<String>,
}
