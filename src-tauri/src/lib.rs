//! AionPartyMeter, derived from A2Tools (GPL-3.0). No remote services.
pub mod capture;
pub mod clock;
pub mod combat;
pub mod entity;
pub mod i18n;
pub mod supporters;
pub mod rederive;
pub mod party;
pub mod pipeline;
#[cfg(feature = "desktop")]
pub mod platform;
#[cfg(feature = "desktop")]
pub mod atomic_file;
#[cfg(feature = "desktop")]
mod desktop;
#[cfg(feature = "desktop")]
pub use desktop::run;
pub mod replay;
