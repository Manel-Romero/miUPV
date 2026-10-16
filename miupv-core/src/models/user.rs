//! User and identity models for the miUPV platform.

use serde::{Deserialize, Serialize};

/// Personal profile data for the authenticated user.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersonData {
    /// Número de Identificación Personal (NIP).
    #[serde(alias = "NIP", alias = "nip", default)]
    pub nip: String,

    /// Given name of the user.
    #[serde(alias = "NOMBRE", alias = "nombre", default)]
    pub nombre: String,

    /// Surnames of the user.
    #[serde(alias = "APELLIDOS", alias = "apellidos", default)]
    pub apellidos: String,

    /// Institutional email address.
    #[serde(alias = "EMAIL", alias = "email", alias = "correo", default)]
    pub email: String,

    /// National Identity Document (DNI/NIE).
    #[serde(alias = "DNI", alias = "dni", default)]
    pub dni: String,

    /// URL to the official photograph.
    #[serde(alias = "FOTO_URL", alias = "foto_url", alias = "fotoUrl", alias = "urlFoto", default)]
    pub foto_url: Option<String>,

    /// User collective type: e.g. "alumno", "docente", "pdi", "pas".
    #[serde(alias = "TIPO", alias = "tipo", alias = "tipo_usuario", alias = "tipoEmp", default)]
    pub tipo: String,

    /// Full name fallback from API (e.g. "name": "APELLIDOS, NOMBRE").
    #[serde(alias = "name", alias = "NAME", default)]
    pub full_name: Option<String>,
}
