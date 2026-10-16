//! Data transfer objects and domain models for miUPV services.

pub mod announcements;
pub mod auth;
pub mod cabinas;
pub mod calendar;
pub mod carnet;
pub mod poliformat;
pub mod scores;
pub mod sports;
pub mod tutorias;
pub mod user;
pub mod utils;
pub mod wol;

pub use announcements::*;
pub use auth::*;
pub use cabinas::*;
pub use calendar::*;
pub use carnet::*;
pub use poliformat::*;
pub use scores::*;
pub use sports::*;
pub use tutorias::*;
pub use user::*;
pub use utils::*;
pub use wol::*;
