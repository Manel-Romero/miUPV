//! Service modules for miUPV APIs.

pub mod announcements_service;
pub mod auth_service;
pub mod cabinas_service;
pub mod calendar_service;
pub mod carnet_service;
pub mod poliformat_service;
pub mod scores_service;
pub mod sports_service;
pub mod tutorias_service;
pub mod user_service;
pub mod wol_service;

// Aliases for Spanish spelling / backwards-compatibility
pub use tutorias_service as tutorías_service;
