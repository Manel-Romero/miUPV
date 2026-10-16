//! # miUPV Core Library
//!
//! Reusable, asynchronous client library for the Universitat Politècnica de València (UPV) services.
//!
//! Provides typed access to:
//! - OAuth 2.0 PKCE authentication and secure keyring persistence
//! - Student grades and statistics (`scores_service`)
//! - PoliformaT (Sakai LMS) sites, assignments, and announcements (`poliformat_service`)
//! - Academic schedules and iCal export (`calendar_service`)
//! - Library study cabin bookings (`cabinas_service`)
//! - Campus sports facility bookings (`sports_service`)
//! - Virtual university ID card (TUI / `carnet_service`)
//! - Laboratory Wake-on-LAN computer awakening (`wol_service`)
//! - Faculty tutoring appointments (`tutorias_service`)
//! - Institutional announcements (`announcements_service`)

pub mod client;
pub mod error;
pub mod models;
pub mod pkce;
pub mod services;

pub use client::{MiupvClient, MiupvClientBuilder, MiupvConfig};
pub use error::CoreError;
pub use models::*;
pub use pkce::{generate_code_challenge, generate_code_verifier};
pub use services::*;
