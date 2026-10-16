//! Academic scores and statistics models.

use super::utils::deserialize_flexible_float_opt;
use serde::{Deserialize, Serialize};

/// Subject grade information for an academic year.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asignatura {
    /// Subject code (e.g. "11542").
    #[serde(alias = "CODIGO", alias = "cod_asi", alias = "codigo_asignatura", alias = "id", alias = "ASI")]
    pub codigo: String,

    /// Subject name.
    #[serde(alias = "NOMBRE", alias = "nom_asi", alias = "nombre_asignatura", alias = "desc", alias = "NOMASI")]
    pub nombre: String,

    /// Exam session / term (e.g. "Ordinaria", "Extraordinaria", "1").
    #[serde(alias = "CONVOCATORIA", alias = "convocatoria", default)]
    pub convocatoria: Option<String>,

    /// Numerical score (e.g. 8.5).
    #[serde(
        alias = "NOTA",
        alias = "nota",
        default,
        deserialize_with = "deserialize_flexible_float_opt"
    )]
    pub nota: Option<f64>,

    /// Textual qualification (e.g. "Aprobado", "Notable", "Sobresaliente", "Suspenso").
    #[serde(alias = "CALIFICACION", alias = "calificacion", alias = "calif", default)]
    pub calificacion: Option<String>,

    /// Number of ECTS credits.
    #[serde(
        alias = "CREDITOS",
        alias = "creditos",
        alias = "cred",
        default,
        deserialize_with = "deserialize_flexible_float_opt"
    )]
    pub creditos: Option<f64>,

    /// Date the score was published or recorded.
    #[serde(alias = "FECHA", alias = "fecha", alias = "fec_eval", alias = "FECHA_PUB", default)]
    pub fecha: Option<String>,
}

/// Aggregated subject statistics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Statistics {
    /// Total enrolled students.
    #[serde(alias = "MATRICULADOS", alias = "matriculados", default)]
    pub matriculados: Option<u32>,

    /// Total students who took the exam.
    #[serde(alias = "PRESENTADOS", alias = "presentados", default)]
    pub presentados: Option<u32>,

    /// Number of passing grades (>= 5.0).
    #[serde(alias = "APROBADOS", alias = "aprobados", default)]
    pub aprobados: Option<u32>,

    /// Number of notables (7.0 - 8.9).
    #[serde(alias = "NOTABLES", alias = "notables", default)]
    pub notables: Option<u32>,

    /// Number of sobresalientes (9.0 - 10.0).
    #[serde(alias = "SOBRESALIENTES", alias = "sobresalientes", default)]
    pub sobresalientes: Option<u32>,

    /// Number of matriculas de honor.
    #[serde(alias = "MATRICULAS_HONOR", alias = "mh", alias = "matriculas_honor", default)]
    pub matriculas_honor: Option<u32>,

    /// Number of failing grades (< 5.0).
    #[serde(alias = "SUSPENSOS", alias = "suspensos", default)]
    pub suspensos: Option<u32>,

    /// Number of students not attending.
    #[serde(alias = "NO_PRESENTADOS", alias = "no_presentados", alias = "np", default)]
    pub no_presentados: Option<u32>,

    /// Average score.
    #[serde(
        alias = "NOTA_MEDIA",
        alias = "nota_media",
        alias = "media",
        default,
        deserialize_with = "deserialize_flexible_float_opt"
    )]
    pub nota_media: Option<f64>,
}
