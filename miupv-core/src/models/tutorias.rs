//! Academic tutoring (tutorías) appointment models.

use crate::models::utils::deserialize_flexible_string;
use serde::{Deserialize, Serialize};

/// Subject offering tutoring sessions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AsignaturaTutoria {
    /// Subject code.
    #[serde(
        alias = "codigo",
        alias = "CODIGO",
        alias = "cod_asi",
        alias = "asi",
        alias = "ASI",
        deserialize_with = "deserialize_flexible_string"
    )]
    pub codigo: String,

    /// Subject name.
    #[serde(
        alias = "nombre",
        alias = "NOMBRE",
        alias = "nom_asi",
        alias = "nomasi",
        alias = "NOMASI"
    )]
    pub nombre: String,

    /// Faculty members teaching this subject.
    #[serde(alias = "profesores", alias = "tutores", alias = "TUTORES", default)]
    pub profesores: Vec<ProfesorTutoria>,
}

/// Professor or instructor available for tutoring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfesorTutoria {
    /// Professor NIP or identifier.
    #[serde(
        alias = "id",
        alias = "nip",
        alias = "NIP",
        alias = "nipTutor",
        alias = "NIPTUTOR",
        deserialize_with = "deserialize_flexible_string"
    )]
    pub id: String,

    /// Full name of the professor.
    #[serde(alias = "nombre", alias = "NOMBRE", alias = "nomTutor", alias = "NOMTUTOR")]
    pub nombre: String,
}

/// An arranged or scheduled tutoring session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tutoria {
    /// Unique tutoring identifier.
    #[serde(alias = "id", alias = "ID", alias = "id_tutoria")]
    pub id: String,

    /// Professor name.
    #[serde(alias = "profesor", alias = "PROFESOR", alias = "nom_prof", default)]
    pub profesor: Option<String>,

    /// Associated subject name or code.
    #[serde(alias = "asignatura", alias = "ASIGNATURA", default)]
    pub asignatura: Option<String>,

    /// Scheduled date.
    #[serde(alias = "fecha", alias = "FECHA", default)]
    pub fecha: Option<String>,

    /// Scheduled time.
    #[serde(alias = "hora", alias = "HORA", default)]
    pub hora: Option<String>,

    /// Session status (e.g. "SOLICITADA", "ACEPTADA", "CANCELADA", "REALIZADA").
    #[serde(alias = "estado", alias = "ESTADO", default)]
    pub estado: Option<String>,

    /// Meeting location (e.g. office number, Teams link).
    #[serde(alias = "lugar", alias = "LUGAR", default)]
    pub lugar: Option<String>,
}

/// Payload sent to request a new tutoring session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolicitudTutoria {
    /// Subject code.
    pub cod_asi: String,

    /// Professor NIP.
    pub nip_profesor: String,

    /// Requested date or timeslot.
    pub fecha: String,

    /// Stated reason or topic for the tutoring.
    pub motivo: String,
}
