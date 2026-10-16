//! University virtual identity card models.

use serde::{Deserialize, Serialize};

/// Summary of the virtual university card (TUI).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CarnetInfo {
    /// NIP of the cardholder.
    #[serde(alias = "nip", alias = "NIP")]
    pub nip: String,

    /// Full name of the cardholder.
    #[serde(alias = "nombre", alias = "NOMBRE")]
    pub nombre: String,

    /// URL to official photograph.
    #[serde(alias = "foto_url", alias = "FOTO_URL", alias = "fotoUrl", default)]
    pub foto_url: Option<String>,

    /// Raw QR code payload for barcode scanning.
    #[serde(
        alias = "qr_data",
        alias = "QR_DATA",
        alias = "qr",
        alias = "cod_qr",
        default
    )]
    pub qr_data: Option<String>,

    /// Card type (e.g. "ESTUDIANTE", "PDI", "PAS", "ALUMNI").
    #[serde(alias = "tipo_carnet", alias = "TIPO_CARNET", alias = "tipo", default)]
    pub tipo_carnet: Option<String>,

    /// Status of the card (e.g. "ACTIVO", "BLOQUEADO").
    #[serde(alias = "estado", alias = "ESTADO", default)]
    pub estado: Option<String>,
}

/// Detailed electronic virtual card record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VCard {
    /// Card identifier.
    #[serde(alias = "id", alias = "ID", default)]
    pub id: Option<String>,

    /// Card type / collective.
    #[serde(alias = "tipo", alias = "TIPO", default)]
    pub tipo: Option<String>,

    /// Card title or description.
    #[serde(alias = "descripcion", alias = "DESCRIPCION", default)]
    pub descripcion: Option<String>,

    /// Physical or electronic card number.
    #[serde(alias = "numero", alias = "NUMERO", default)]
    pub numero: Option<String>,

    /// QR code payload.
    #[serde(alias = "qr_data", alias = "QR_DATA", default)]
    pub qr_data: Option<String>,
}
